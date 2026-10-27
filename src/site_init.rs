//! `rpress site init` — onboard a WP source into `sites/<name>.env`.
//!
//! Three sources, all probed rather than assumed:
//!   ssh     — find wp-config.php on the host, read DB creds, list/create dumps
//!   local   — scan a dir for *.sql + wp-content/
//!   archive — extract .zip/.tar/.sql.gz into work/<site>/backup-src, then local
//!
//! Every question has a flag twin so agents can run fully non-interactive.

use crate::site::{self, SiteEnv};
use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct InitArgs {
    pub site: Option<String>,
    pub source: Option<String>,
    pub ssh: Option<String>,
    pub wp_root: Option<String>,
    pub dump_remote: Option<String>,
    pub fresh_dump: bool,
    pub dump: Option<PathBuf>,
    pub wp_content: Option<PathBuf>,
    pub backup: Option<PathBuf>,
    pub site_url: Option<String>,
    pub bind: Option<String>,
    pub name: Option<String>,
    pub tour_base: Option<String>,
    pub clone_ok: bool,
    pub force: bool,
}

pub fn run(a: InitArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let site = site::need(a.site.clone(), "site", "Site name (slug)", None)?;
    if !site::valid_slug(&site) {
        bail!("invalid site slug {site:?} — want [a-z0-9][a-z0-9-]*");
    }
    let env_file = site::env_path(&root, &site);
    if env_file.exists() && !a.force {
        if site::tty() {
            if !site::confirm(&format!("{} exists — overwrite?", env_file.display()), false)? {
                bail!("aborted");
            }
        } else {
            bail!("{} exists — pass --force to overwrite", env_file.display());
        }
    }

    let source = match a.source.as_deref() {
        Some(s) => s.to_string(),
        None if site::tty() => {
            let items = vec![
                "ssh     — probe a VPS for wp-config + dump".to_string(),
                "local   — a .sql dump (+ wp-content dir) already on disk".to_string(),
                "archive — a .zip/.tar(.gz)/.sql.gz backup to unpack".to_string(),
            ];
            ["ssh", "local", "archive"][site::choose("Source", &items)?].to_string()
        }
        None => bail!("missing --source ssh|local|archive"),
    };

    let mut env = SiteEnv {
        site: site.clone(),
        name: a.name.clone().unwrap_or_else(|| site.clone()),
        host: "127.0.0.1".into(),
        tour_base: a.tour_base.clone().unwrap_or_else(|| "voyage".into()),
        ..Default::default()
    };

    let mut probed_url: Option<String> = None;
    match source.as_str() {
        "ssh" => ssh_flow(&a, &root, &mut env, &mut probed_url)?,
        "local" => local_flow(&a, &root, &site, &mut env, &mut probed_url)?,
        "archive" => archive_flow(&a, &root, &site, &mut env, &mut probed_url)?,
        s => bail!("unknown --source {s:?} — ssh|local|archive"),
    }

    // origin URL — probe result is only a default, always confirmable
    env.site_url = site::need(
        a.site_url.clone().or(probed_url),
        "site-url",
        "Origin site URL",
        None,
    )?;

    // bind — flag, or first free port from 8410 upward
    env.bind = match a.bind.clone() {
        Some(b) => b,
        None if site::tty() => site::prompt("Local bind", Some(&free_bind()))?,
        None => free_bind(),
    };
    env.local_url = format!("http://localhost:{}", env.port());

    // clone guard — remote pull must never touch production by accident
    if env.has_remote() {
        env.clone_source_ok = a.clone_ok
            || (site::tty()
                && site::confirm(
                    "Is this source a CLONE/STAGING (never production)?",
                    false,
                )?);
        if !env.clone_source_ok {
            bail!("refused: pass --clone-ok only for clone/staging sources");
        }
    } else {
        // local files are user-supplied; nothing remote can be hurt
        env.clone_source_ok = true;
    }

    let path = env.save(&root)?;
    eprintln!("\nwrote {}", path.display());
    eprintln!(
        "next:\n  rpress site build {site}\n  rpress site serve {site}"
    );
    Ok(())
}

fn free_bind() -> String {
    for p in 8410..8460 {
        if TcpListener::bind(("127.0.0.1", p)).is_ok() {
            return format!("127.0.0.1:{p}");
        }
    }
    "127.0.0.1:8410".into()
}

// ---------- ssh source ----------

struct WpCreds {
    db_name: String,
    db_user: String,
    db_pass: String,
    db_host: String,
    prefix: String,
    has_wp: bool,
    has_mysqldump: bool,
    has_mysql: bool,
}

fn ssh_flow(a: &InitArgs, root: &Path, env: &mut SiteEnv, probed_url: &mut Option<String>) -> Result<()> {
    let target = site::need(a.ssh.clone(), "ssh", "SSH target (user@host or vps.sh alias)", None)?;
    let remote = site::Remote::resolve(&target, root);
    eprintln!("probing {target} …");
    let t = remote.exec("echo ok")?;
    if !t.ok {
        bail!(
            "cannot reach {target}: {}\nfix: ssh key in agent / ~/.ssh/config, or a vps.sh alias in ../vps-ssh",
            t.stderr.trim()
        );
    }

    // wp-config.php → wp_root
    let wp_root = match a.wp_root.clone() {
        Some(r) => r,
        None => {
            eprintln!("searching wp-config.php …");
            let hits = remote.lines(
                "find /var/www /srv/www /usr/local/www /home /opt ~ -maxdepth 6 -name wp-config.php 2>/dev/null | head -30",
            )?;
            if hits.is_empty() {
                bail!("no wp-config.php found under /var/www /srv/www /home /opt ~ — pass --wp-root");
            }
            let dirs: Vec<String> = hits
                .iter()
                .map(|p| p.rsplit_once('/').map(|(d, _)| d).unwrap_or(p).to_string())
                .collect();
            dirs[site::choose("WordPress root", &dirs)?].clone()
        }
    };
    eprintln!("wp_root = {wp_root}");

    // DB creds from wp-config.php
    let creds = wp_creds(&remote, &wp_root)?;
    eprintln!(
        "db = {} (prefix {}, host {}){}",
        creds.db_name,
        creds.prefix,
        creds.db_host,
        if creds.has_wp { " +wp-cli" } else { "" }
    );

    // dump — existing file or fresh export
    let dump_remote = if let Some(d) = a.dump_remote.clone() {
        d
    } else if a.fresh_dump {
        export_dump(&remote, &wp_root, &creds)?
    } else {
        let found = remote.lines(
            "find ~ /tmp /var /srv -maxdepth 4 \\( -name '*.sql' -o -name '*.sql.gz' \\) -size +1M -exec du -h {} + 2>/dev/null | sort -rh | head -20",
        )?;
        let mut opts: Vec<String> = found
            .iter()
            .map(|l| l.split_once('\t').map(|(s, p)| format!("{p}  ({s})")).unwrap_or_else(|| l.clone()))
            .collect();
        if creds.has_wp || creds.has_mysqldump {
            opts.push("<create a fresh dump now>".into());
        }
        if found.is_empty() && !creds.has_wp && !creds.has_mysqldump {
            bail!("no *.sql on host and no wp-cli/mysqldump — pass --dump-remote");
        }
        let i = site::choose("SQL dump", &opts)?;
        if i < found.len() {
            found[i]
                .split_once('\t')
                .map(|(_, p)| p.to_string())
                .unwrap_or_else(|| found[i].clone())
        } else {
            export_dump(&remote, &wp_root, &creds)?
        }
    };
    eprintln!("dump = {dump_remote}");

    // siteurl from the live DB (best effort — user can override anyway)
    if creds.has_mysql {
        if let Some(u) = probe_siteurl(&remote, &creds) {
            *probed_url = Some(u);
        }
    }

    match remote.env_pair() {
        ("VPS_HOST", v) => env.vps_host = Some(v),
        (_, v) => env.ssh_target = Some(v),
    }
    env.wp_root = Some(wp_root);
    env.dump_remote = Some(dump_remote);
    Ok(())
}

/// Whole-expression quoted literal? `'x'` or `"x"` — anything else
/// (concat `'a'.'b'`, constants, calls) is None, never a partial match.
fn php_str_literal(expr: &str) -> Option<String> {
    regex::Regex::new(r#"^'([^']*)'$|^"([^"]*)"$"#)
        .unwrap()
        .captures(expr)
        .map(|c| {
            c.get(1)
                .or_else(|| c.get(2))
                .unwrap()
                .as_str()
                .to_string()
        })
}

/// wp-config-derived strings are attacker-influenced data. Every value that
/// reaches a remote shell string is shell-quoted AND charset-checked — a
/// credential outside `[A-Za-z0-9_.$-]` is a config bug or an attack, never
/// a legitimate WordPress value. `extra` widens the set for DB_HOST, where
/// `host:port` and unix-socket paths are real.
fn check_cred(what: &str, v: &str, extra: &[u8]) -> Result<()> {
    let ok = !v.is_empty()
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || extra.contains(&b));
    if !ok {
        bail!(
            "wp-config.php {what} value {v:?} contains characters outside [A-Za-z0-9{}] — refusing to use it in remote commands",
            String::from_utf8_lossy(extra)
        );
    }
    Ok(())
}

/// Escape a value for a double-quoted my.cnf option-file string — `\\` and
/// `\"` are the only metacharacters mysql parses inside double quotes there.
fn my_cnf_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn wp_creds(remote: &site::Remote, wp_root: &str) -> Result<WpCreds> {
    let q = site::shell_quote(&format!("{wp_root}/wp-config.php"));
    let o = remote.exec(
        &format!(
            "grep -E 'DB_NAME|DB_USER|DB_PASSWORD|DB_HOST|table_prefix' {q}; for c in wp mysqldump mysql; do command -v $c >/dev/null && echo HAVE_$c; done"
        ),
    )?;
    if !o.ok {
        bail!("cannot read {wp_root}/wp-config.php: {}", o.stderr.trim());
    }
    // define('KEY', <expr>) — capture the WHOLE expression so trailing
    // concatenation/constants fail loudly instead of partially matching.
    // The key itself must be a quoted token — `XDB_NAME`/`DB_NAMEX`
    // defines must not shadow the real constants.
    let def = |key: &str| -> Result<Option<String>> {
        let re = regex::Regex::new(&format!(r#"['\"]{key}['\"]\s*,\s*(.+?)\s*\)"#)).unwrap();
        match re.captures(&o.stdout) {
            None => Ok(None),
            Some(cap) => php_str_literal(cap[1].trim()).map(Some).ok_or_else(|| {
                anyhow::anyhow!(
                    "wp-config.php {key} is not a simple quoted literal ({} — concat/constant?) — fix wp-config or pass values manually",
                    cap[1].trim()
                )
            }),
        }
    };
    // $table_prefix = <expr>;  — same whole-literal rule
    let prefix = match regex::Regex::new(r#"\$table_prefix\s*=\s*(.+?)\s*;"#)
        .unwrap()
        .captures(&o.stdout)
    {
        None => "wp_".into(),
        Some(cap) => php_str_literal(cap[1].trim()).with_context(|| {
            format!(
                "wp-config.php $table_prefix is not a quoted literal ({}) — fix wp-config",
                cap[1].trim()
            )
        })?,
    };
    let mut c = WpCreds {
        db_name: def("DB_NAME")?.unwrap_or_default(),
        db_user: def("DB_USER")?.unwrap_or_default(),
        db_pass: def("DB_PASSWORD")?.unwrap_or_default(),
        db_host: def("DB_HOST")?.unwrap_or_default(),
        prefix,
        has_wp: o.stdout.contains("HAVE_wp"),
        has_mysqldump: o.stdout.contains("HAVE_mysqldump"),
        has_mysql: o.stdout.contains("HAVE_mysql"),
    };
    if c.db_name.is_empty() {
        bail!("wp-config.php has no DB_NAME — not a WordPress root?");
    }
    if c.db_host.is_empty() {
        c.db_host = "localhost".into();
    }
    check_cred("DB_NAME", &c.db_name, b"._$-")?;
    check_cred("DB_USER", &c.db_user, b"._$-")?;
    // host:port and socket paths are legal DB_HOSTs — allow : and /
    check_cred("DB_HOST", &c.db_host, b"._$/:-")?;
    check_cred("table_prefix", &c.prefix, b"._$-")?;
    if c.db_pass.contains(['\n', '\r']) {
        bail!("DB_PASSWORD contains a newline — cannot be carried in a my.cnf handoff");
    }
    Ok(c)
}

/// Write a temp `[client]` defaults file on the remote holding the DB
/// password. A quoted heredoc feeds `cat` — the password bytes ride inside
/// the ssh command payload (a `cat` that exists for milliseconds), never on
/// the mysql/mysqldump process line where `ps` would expose them for the
/// whole query/export duration.
fn remote_my_cnf(remote: &site::Remote, c: &WpCreds) -> Result<String> {
    let body = format!(
        "[client]\nuser={}\npassword=\"{}\"\nhost={}\n",
        c.db_user,
        my_cnf_escape(&c.db_pass),
        c.db_host
    );
    let cmd = format!(
        "f=$(mktemp /tmp/rpress-my.XXXXXX.cnf) && chmod 600 \"$f\" && cat >\"$f\" <<'RPRESS_CNF'\n{body}RPRESS_CNF\nprintf %s \"$f\""
    );
    remote
        .lines(&cmd)?
        .into_iter()
        .next()
        .context("no defaults-file path back from mktemp")
}

/// Run `body` on the remote while `cnf` exists, then remove the defaults
/// file — pass or fail — before returning.
fn run_my_cnf(remote: &site::Remote, cnf: &str, body: &str) -> Result<site::Out> {
    let cmd = format!("{{ {body}; rc=$?; rm -f {}; exit $rc; }}", site::shell_quote(cnf));
    match remote.exec(&cmd) {
        Ok(o) => Ok(o),
        Err(e) => {
            let _ = remote.exec(&format!("rm -f {}", site::shell_quote(cnf)));
            Err(e)
        }
    }
}

/// Best-effort siteurl probe through the defaults-file path — `MYSQL_PWD`
/// never appears in the remote command line.
fn probe_siteurl(remote: &site::Remote, c: &WpCreds) -> Option<String> {
    let cnf = remote_my_cnf(remote, c).ok()?;
    let sql = format!(
        "SELECT option_value FROM {}options WHERE option_name='siteurl'",
        c.prefix
    );
    let o = run_my_cnf(
        remote,
        &cnf,
        &format!(
            "mysql --defaults-extra-file={} -B -N -h{} -u{} -e {} {}",
            site::shell_quote(&cnf),
            site::shell_quote(&c.db_host),
            site::shell_quote(&c.db_user),
            site::shell_quote(&sql),
            site::shell_quote(&c.db_name)
        ),
    )
    .ok()?;
    if !o.ok {
        return None;
    }
    o.stdout
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

fn export_dump(remote: &site::Remote, wp_root: &str, c: &WpCreds) -> Result<String> {
    eprintln!("exporting dump on host …");
    // resolve $HOME first — a quoted path gets no ~ expansion from the shell
    let home = remote
        .lines("printf %s \"$HOME\"")?
        .into_iter()
        .next()
        .context("cannot resolve remote $HOME")?;
    let path = format!("{home}/rpress-{}-{}.sql", c.db_name, epoch());
    if c.has_wp {
        let cmd = format!(
            "wp --path={} db export {} --allow-root",
            site::shell_quote(wp_root),
            site::shell_quote(&path)
        );
        let o = remote.exec(&cmd)?;
        if !o.ok {
            bail!("remote dump export failed: {}", o.stderr.trim());
        }
    } else {
        if !c.has_mysqldump {
            bail!("no wp-cli or mysqldump on host — create the dump manually and pass --dump-remote");
        }
        let cnf = remote_my_cnf(remote, c)?;
        let o = run_my_cnf(
            remote,
            &cnf,
            &format!(
                "mysqldump --defaults-extra-file={} {} > {}",
                site::shell_quote(&cnf),
                site::shell_quote(&c.db_name),
                site::shell_quote(&path)
            ),
        )?;
        if !o.ok {
            bail!("remote dump export failed: {}", o.stderr.trim());
        }
    }
    let abs = remote.lines(&format!("ls {}", site::shell_quote(&path)))?;
    Ok(abs.into_iter().next().unwrap_or(path))
}

fn epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------- local source ----------

fn local_flow(
    a: &InitArgs,
    root: &Path,
    site_name: &str,
    env: &mut SiteEnv,
    probed_url: &mut Option<String>,
) -> Result<()> {
    let (dump, wp_content) = match (&a.dump, &a.wp_content) {
        (Some(d), Some(w)) => (d.clone(), Some(w.clone())),
        _ => {
            // scan a starting dir: --dump dir | --wp-content parent | cwd
            let start = a
                .dump
                .clone()
                .filter(|p| p.is_dir())
                .or_else(|| a.wp_content.clone())
                .unwrap_or_else(|| root.to_path_buf());
            let probe = probe_local(&start);
            let dump = match a.dump.clone().filter(|p| p.is_file()) {
                Some(d) => d,
                None => {
                    if probe.dumps.is_empty() {
                        bail!("no *.sql under {} — pass --dump <file>", start.display());
                    }
                    let opts: Vec<String> = probe
                        .dumps
                        .iter()
                        .map(|(p, s)| format!("{}  ({})", p.display(), human(*s)))
                        .collect();
                    probe.dumps[site::choose("SQL dump", &opts)?].0.clone()
                }
            };
            let wc = match a.wp_content.clone().filter(|p| p.is_dir()) {
                Some(w) if w.file_name().map(|n| n == "wp-content").unwrap_or(false) => Some(w),
                Some(w) if w.join("wp-content").is_dir() => Some(w.join("wp-content")),
                _ => {
                    if probe.wp_contents.is_empty() {
                        eprintln!("WARN no wp-content dir found — assets will be missing");
                        None
                    } else {
                        let opts: Vec<String> =
                            probe.wp_contents.iter().map(|p| p.display().to_string()).collect();
                        Some(probe.wp_contents[site::choose("wp-content dir", &opts)?].clone())
                    }
                }
            };
            (dump, wc)
        }
    };

    check_dump(&dump)?;
    *probed_url = detect_siteurl(&dump);
    env.dump_local = Some(absolutize(&dump)?);
    env.wp_content_local = wp_content.map(|p| absolutize(&p)).transpose()?;
    let _ = site_name;
    Ok(())
}

fn absolutize(p: &Path) -> Result<PathBuf> {
    Ok(std::fs::canonicalize(p).with_context(|| format!("resolve {}", p.display()))?)
}

struct LocalProbe {
    /// (path, bytes) — sorted largest first
    dumps: Vec<(PathBuf, u64)>,
    wp_contents: Vec<PathBuf>,
}

fn probe_local(start: &Path) -> LocalProbe {
    let mut p = LocalProbe { dumps: vec![], wp_contents: vec![] };
    walk(start, 0, &mut |path, size| {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name == "wp-content"
            && path.is_dir()
            && ["uploads", "themes", "plugins"].iter().any(|s| path.join(s).is_dir())
        {
            p.wp_contents.push(path.to_path_buf());
        } else if (name.ends_with(".sql") || name.ends_with(".sql.gz")) && size > 1_000_000 {
            p.dumps.push((path.to_path_buf(), size));
        }
    });
    p.dumps.sort_by(|a, b| b.1.cmp(&a.1));
    p
}

fn walk(dir: &Path, depth: u8, f: &mut impl FnMut(&Path, u64)) {
    if depth > 6 {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let path = e.path();
        let Ok(meta) = e.metadata() else { continue };
        f(&path, meta.len());
        if meta.is_dir() {
            walk(&path, depth + 1, f);
        }
    }
}

fn human(n: u64) -> String {
    if n > 1 << 30 {
        format!("{:.1}G", n as f64 / (1 << 30) as f64)
    } else if n > 1 << 20 {
        format!("{}M", n >> 20)
    } else {
        format!("{}K", n >> 10)
    }
}

/// `zcat <path> | head -c <n>` without a shell: spawn zcat, read at most `n`
/// bytes, then kill the child — dropping a pipe mid-write only yields the
/// silent SIGPIPE for zcat, and `head`'s early-exit semantics stay intact.
fn gz_head(path: &Path, n: u64) -> Result<Vec<u8>> {
    let mut child = Command::new("zcat")
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("spawn zcat {}", path.display()))?;
    let mut buf = Vec::new();
    let res = child
        .stdout
        .take()
        .context("zcat stdout")?
        .take(n)
        .read_to_end(&mut buf);
    let _ = child.kill();
    let _ = child.wait();
    res.with_context(|| format!("zcat {}", path.display()))?;
    Ok(buf)
}

fn check_dump(dump: &Path) -> Result<()> {
    let head: Vec<u8> = if dump.extension().map(|e| e == "gz").unwrap_or(false) {
        // only the first 256KB — never decompress the whole dump into RAM
        gz_head(dump, 256 * 1024)?
    } else {
        let mut buf = vec![0u8; 256 * 1024];
        let n = std::fs::File::open(dump)
            .with_context(|| format!("open {}", dump.display()))?
            .read(&mut buf)?;
        buf.truncate(n);
        buf
    };
    let text = String::from_utf8_lossy(&head);
    if !text.contains("CREATE TABLE") && !text.contains("INSERT INTO") {
        bail!("{} does not look like a SQL dump", dump.display());
    }
    Ok(())
}

fn detect_siteurl(dump: &Path) -> Option<String> {
    let re = regex::Regex::new(r#"'siteurl',\s*'((?:[^'\\]|\\.)*)'"#).ok()?;
    let grab = |text: &str| re.captures(text).map(|c| c[1].to_string());
    // stream until first hit — dumps are big, never read them whole.
    // For .gz the producer is a zcat child (argv-passed path — no shell,
    // no interpolation) we kill on the first match or at the line cap.
    let mut child;
    let gz = dump.extension().map(|e| e == "gz").unwrap_or(false);
    let reader: Box<dyn BufRead> = if gz {
        let mut ch = Command::new("zcat")
            .arg(dump)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        let out = ch.stdout.take()?;
        child = Some(ch);
        Box::new(BufReader::new(out))
    } else {
        child = None;
        Box::new(BufReader::new(std::fs::File::open(dump).ok()?))
    };
    // plain files cap at 5000 lines; the gz scan (old `grep -m1`) runs to
    // first match or stream end — a late siteurl must still be found.
    // The child is killed+reaped on every exit path.
    let cap = if gz { usize::MAX } else { 5000 };
    let mut found = None;
    for line in reader.lines().take(cap) {
        match line {
            Ok(line) => {
                if let Some(u) = grab(&line) {
                    found = Some(u);
                    break;
                }
            }
            // non-UTF8 line (latin1 dumps) — skip it, don't give up the scan
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => continue,
            Err(_) => break,
        }
    }
    if let Some(ch) = &mut child {
        let _ = ch.kill();
        let _ = ch.wait();
    }
    found
}

// ---------- archive source ----------

fn archive_flow(
    a: &InitArgs,
    root: &Path,
    site_name: &str,
    env: &mut SiteEnv,
    probed_url: &mut Option<String>,
) -> Result<()> {
    let backup = a.backup.clone().or_else(|| a.dump.clone());
    let backup = match backup {
        Some(b) => b,
        None => {
            let b = site::need(None, "backup", "Backup archive path", None)?;
            PathBuf::from(b)
        }
    };
    if !backup.is_file() {
        bail!("backup not found: {}", backup.display());
    }
    let dest = root.join("work").join(site_name).join("backup-src");
    std::fs::create_dir_all(&dest)?;
    let name = backup.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    eprintln!("extracting {} → {}", backup.display(), dest.display());
    let ok = if name.ends_with(".zip") {
        site::run(Command::new("unzip").args(["-q", "-o"]).arg(&backup).arg("-d").arg(&dest))?.ok
    } else if name.ends_with(".tar") || name.contains(".tar.") || name.ends_with(".tgz") {
        site::run(Command::new("tar").arg("xf").arg(&backup).arg("-C").arg(&dest))?.ok
    } else if name.ends_with(".sql.gz") {
        let out = dest.join("dump.sql");
        let f = std::fs::File::create(&out)
            .with_context(|| format!("create {}", out.display()))?;
        // `gunzip -c` straight into the file — argv only, no shell
        site::run(Command::new("gunzip").args(["-c"]).arg(&backup).stdout(std::process::Stdio::from(f)))?.ok
    } else if name.ends_with(".sql") {
        env.dump_local = Some(backup.clone());
        true
    } else {
        bail!("unsupported archive type: {name} (want .zip/.tar/.tar.gz/.sql.gz/.sql)");
    };
    if !ok {
        bail!("extract failed: {}", backup.display());
    }
    if name.ends_with(".sql") {
        check_dump(&backup)?;
        *probed_url = detect_siteurl(&backup);
        return Ok(());
    }
    // treat extracted tree as a local source
    let probe = probe_local(&dest);
    if probe.dumps.is_empty() {
        bail!("no *.sql inside {}", backup.display());
    }
    let opts: Vec<String> = probe.dumps.iter().map(|(p, s)| format!("{}  ({})", p.display(), human(*s))).collect();
    let dump = probe.dumps[site::choose("SQL dump", &opts)?].0.clone();
    check_dump(&dump)?;
    *probed_url = detect_siteurl(&dump);
    env.dump_local = Some(dump);
    env.wp_content_local = probe.wp_contents.into_iter().next();
    if env.wp_content_local.is_none() {
        eprintln!("WARN no wp-content dir inside archive — assets will be missing");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_str_literal_parses_both_quote_styles() {
        assert_eq!(php_str_literal("'wp_'").as_deref(), Some("wp_"));
        assert_eq!(php_str_literal("\"dbname\"").as_deref(), Some("dbname"));
        assert_eq!(php_str_literal("''").as_deref(), Some(""));
        // whole-value anchor: trailing expressions never partially match
        assert_eq!(php_str_literal("'a'.'b'"), None);
        assert_eq!(php_str_literal("'a' . 'b'"), None);
        assert_eq!(php_str_literal("DB_EXTRA"), None);
        assert_eq!(php_str_literal("getenv('DB')"), None);
        assert_eq!(php_str_literal("'a'; echo"), None);
    }

    /// Same regex pair wp_creds uses against grepped wp-config lines.
    fn wp_define(text: &str, key: &str) -> Result<Option<String>> {
        let re = regex::Regex::new(&format!(r#"['\"]{key}['\"]\s*,\s*(.+?)\s*\)"#)).unwrap();
        match re.captures(text) {
            None => Ok(None),
            Some(cap) => php_str_literal(cap[1].trim())
                .map(Some)
                .ok_or_else(|| anyhow::anyhow!("{key} not literal: {}", cap[1].trim())),
        }
    }

    #[test]
    fn wp_define_accepts_single_and_double_quotes() {
        let cfg = "define( 'DB_NAME', 'wordpress' );\ndefine('DB_USER', \"wp_user\");\n";
        assert_eq!(wp_define(cfg, "DB_NAME").unwrap().as_deref(), Some("wordpress"));
        assert_eq!(wp_define(cfg, "DB_USER").unwrap().as_deref(), Some("wp_user"));
        assert_eq!(wp_define(cfg, "DB_HOST").unwrap(), None);
    }

    #[test]
    fn wp_define_ignores_lookalike_keys() {
        // key must be a quoted token — XDB_NAME / DB_NAMEX must not shadow
        let cfg = "define('XDB_NAME', 'evil');\ndefine('DB_NAMEX', 'evil2');\ndefine('DB_NAME', 'real_db');\n";
        assert_eq!(wp_define(cfg, "DB_NAME").unwrap().as_deref(), Some("real_db"));
        // and when only lookalikes exist, the key is absent, not "evil"
        assert_eq!(wp_define("define('XDB_NAME', 'evil');", "DB_NAME").unwrap(), None);
    }

    #[test]
    fn wp_define_rejects_trailing_expression() {
        for cfg in [
            "define('DB_NAME', 'base' . '_db');",
            "define('DB_NAME', SOME_CONST);",
            "define('DB_NAME', 'x'); define('DB_USER', 'u'); //'junk');", // greedy tail must not steal
        ] {
            // either an error or a clean parse — never a partial prefix match
            match wp_define(cfg, "DB_NAME") {
                Ok(Some(v)) => assert!(v == "x" || v == "wordpress" || v.contains('b'), "v={v}"),
                Ok(None) | Err(_) => {}
            }
        }
        // the dangerous shapes must error outright
        assert!(wp_define("define('DB_NAME', 'a'.'b');", "DB_NAME").is_err());
        assert!(wp_define("define('DB_NAME', CONST_EXPR);", "DB_NAME").is_err());
    }

    #[test]
    fn cred_charset_validation() {
        check_cred("DB_USER", "wp_user-1.x", b"._$-").unwrap();
        check_cred("table_prefix", "wp_$", b"._$-").unwrap();
        for bad in ["root';id;#", "a|b", "a`b`", "a b", "a\nb"] {
            assert!(check_cred("DB_USER", bad, b"._$-").is_err(), "{bad}");
        }
        // DB_HOST's wider set still keeps shell metachars out
        check_cred("DB_HOST", "localhost:3306", b"._$/:-").unwrap();
        check_cred("DB_HOST", "/run/mysqld.sock", b"._$/:-").unwrap();
        assert!(check_cred("DB_HOST", "h';rm -rf /;'", b"._$/:-").is_err());
    }

    #[test]
    fn my_cnf_escapes_quotes_and_backslashes() {
        assert_eq!(my_cnf_escape("plain"), "plain");
        assert_eq!(my_cnf_escape("p\"w"), "p\\\"w");
        assert_eq!(my_cnf_escape("a\\b"), "a\\\\b");
    }
}
