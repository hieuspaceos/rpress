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
        if let Ok(lines) = remote.lines(
            &format!(
                "MYSQL_PWD='{}' mysql -B -N -h'{}' -u'{}' -e \"SELECT option_value FROM {}options WHERE option_name='siteurl'\" '{}'",
                creds.db_pass.replace('\'', "'\\''"),
                creds.db_host,
                creds.db_user,
                creds.prefix,
                creds.db_name
            ),
        ) {
            *probed_url = lines.into_iter().next();
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

fn wp_creds(remote: &site::Remote, wp_root: &str) -> Result<WpCreds> {
    let o = remote.exec(
        &format!(
            "grep -E 'DB_NAME|DB_USER|DB_PASSWORD|DB_HOST|table_prefix' '{wp_root}/wp-config.php'; for c in wp mysqldump mysql; do command -v $c >/dev/null && echo HAVE_$c; done"
        ),
    )?;
    if !o.ok {
        bail!("cannot read {wp_root}/wp-config.php: {}", o.stderr.trim());
    }
    let val = |key: &str| -> String {
        let re = regex::Regex::new(&format!(r#"{key}'?\s*,\s*'([^']*)'"#)).unwrap();
        re.captures(&o.stdout)
            .map(|c| c[1].to_string())
            .unwrap_or_default()
    };
    let prefix = regex::Regex::new(r#"table_prefix\s*=\s*'([^']*)'"#)
        .unwrap()
        .captures(&o.stdout)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "wp_".into());
    let c = WpCreds {
        db_name: val("DB_NAME"),
        db_user: val("DB_USER"),
        db_pass: val("DB_PASSWORD"),
        db_host: if val("DB_HOST").is_empty() { "localhost".into() } else { val("DB_HOST") },
        prefix,
        has_wp: o.stdout.contains("HAVE_wp"),
        has_mysqldump: o.stdout.contains("HAVE_mysqldump"),
        has_mysql: o.stdout.contains("HAVE_mysql"),
    };
    if c.db_name.is_empty() {
        bail!("wp-config.php has no DB_NAME — not a WordPress root?");
    }
    Ok(c)
}

fn export_dump(remote: &site::Remote, wp_root: &str, c: &WpCreds) -> Result<String> {
    eprintln!("exporting dump on host …");
    let path = format!("~/rpress-{}-{}.sql", c.db_name, epoch());
    let cmd = if c.has_wp {
        format!("wp --path='{wp_root}' db export '{path}' --allow-root")
    } else {
        format!(
            "MYSQL_PWD='{}' mysqldump -h'{}' -u'{}' '{}' > '{path}'",
            c.db_pass.replace('\'', "'\\''"),
            c.db_host,
            c.db_user,
            c.db_name
        )
    };
    let o = remote.exec(&cmd)?;
    if !o.ok {
        bail!("remote dump export failed: {}", o.stderr.trim());
    }
    // expand ~ on the remote side
    let abs = remote.lines(&format!("ls {}", path.replace('~', "$HOME")))?;
    Ok(abs.into_iter().next().unwrap_or_else(|| path.replace('~', "/root")))
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

fn check_dump(dump: &Path) -> Result<()> {
    use std::io::Read;
    let head: Vec<u8> = if dump.extension().map(|e| e == "gz").unwrap_or(false) {
        // only the first 256KB — never decompress the whole dump into RAM
        let o = site::run(Command::new("sh").args([
            "-c",
            &format!("zcat '{}' | head -c 262144", dump.display()),
        ]))?;
        o.stdout.into_bytes()
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
    if dump.extension().map(|e| e == "gz").unwrap_or(false) {
        let o = site::run(Command::new("sh").args(["-c", &format!("zcat '{}' | grep -m1 siteurl", dump.display())])).ok()?;
        grab(&o.stdout)
    } else {
        // stream until first hit — dumps are big, don't read them whole
        use std::io::{BufRead, BufReader};
        let f = std::fs::File::open(dump).ok()?;
        for line in BufReader::new(f).lines().take(5000) {
            let line = line.ok()?;
            if let Some(u) = grab(&line) {
                return Some(u);
            }
        }
        None
    }
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
        let o = site::run(Command::new("sh").args([
            "-c",
            &format!("gunzip -c '{}' > '{}'", backup.display(), out.display()),
        ]))?;
        o.ok
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
