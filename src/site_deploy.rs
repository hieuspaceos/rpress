//! `rpress site deploy` — push a tested sandbox site to a real target.
//!
//! Three targets, always behind an explicit --domain (never guessed from the
//! source origin — that is how production accidents happen):
//!
//!   static  — regenerate with the public domain, rsync cache/+www/ to a
//!             remote docroot (or a local dir). Dynamic routes die: no admin,
//!             no comments, no intake — the honest trade-off, printed upfront.
//!   pages   — same static output, pushed to Cloudflare Pages via wrangler.
//!   vps     — full app: rsync the binary + rpress.db + www/ to a remote
//!             app dir, plus a ready nginx server-block + systemd unit the
//!             user installs once. Dynamic features stay alive.
//!
//! `--dry-run` prints exactly what would happen without touching anything.

use crate::db::conn;
use crate::generate;
use crate::site::{self, SiteEnv};
#[allow(unused_imports)]
use crate::sql;
use anyhow::{bail, Context, Result};
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct DeployArgs {
    pub site: String,
    /// static | pages | vps
    pub target: String,
    /// public origin, e.g. https://example.com — mandatory, never defaulted
    pub domain: Option<String>,
    /// static/vps: destination — dir | user@host:/path | vps:alias:/path
    pub dest: Option<String>,
    /// pages: Cloudflare Pages project name (default: site slug)
    pub project: Option<String>,
    /// vps: remote app dir (default: the dest path itself)
    pub app_dir: Option<String>,
    /// skip the confirmation prompt (agents)
    pub yes: bool,
    pub dry_run: bool,
}

pub fn run(a: DeployArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let env = SiteEnv::load(&root, &a.site)?;
    let work = root.join("work").join(&env.site);
    let db = work.join("rpress.db");
    let cache = work.join("cache");

    if !db.is_file() || !cache.join("index.html").is_file() {
        bail!("site not built — run `rpress site build {}` first", env.site);
    }
    let domain = site::need(
        a.domain.clone(),
        "domain",
        "Public domain (https://…)",
        None,
    )?;
    let domain = domain.trim_end_matches('/').to_string();
    if !valid_origin(&domain) {
        bail!("--domain must be a full origin like https://example.com");
    }

    if domain == env.site_url.trim_end_matches('/') {
        bail!("--domain equals the source origin — deploying onto the clone is almost certainly wrong");
    }

    eprintln!("deploy plan\n  site    {}\n  target  {}\n  domain  {domain}", env.site, a.target);
    match a.target.as_str() {
        "static" => eprintln!("  note    static export — admin/comments/intake will NOT work on the target"),
        "vps" => eprintln!("  note    full app — binary + db + assets, needs the printed nginx+systemd setup"),
        _ => {}
    }
    if !a.yes && !a.dry_run {
        if site::tty() {
            if !site::confirm("proceed?", false)? {
                bail!("aborted");
            }
        } else {
            bail!("non-interactive deploy needs --yes (or --dry-run to preview)");
        }
    }

    match a.target.as_str() {
        "static" => deploy_static(&a, &root, &env, &work, &domain),
        "pages" => deploy_pages(&a, &root, &env, &work, &domain),
        "vps" => deploy_vps(&a, &root, &env, &work, &domain),
        t => bail!("unknown --target {t:?} — static|pages|vps"),
    }
}

// ---------- destination ----------

/// `--dest` parses to a transport + a base path. `vps:` prefix picks the
/// vps.sh askpass channel; `host:` is plain ssh; a bare path is local.
enum Dest {
    Local { base: PathBuf },
    Ssh { target: String, base: String },
    Vps { userip: String, port: String, askpass: PathBuf, base: String },
}

fn parse_dest(dest: &str, root: &Path) -> Result<Dest> {
    if let Some(rest) = dest.strip_prefix("vps:") {
        let (name, path) = rest.split_once(':').context("--dest vps:<alias>:<path>")?;
        let (userip, port) = vps_addr(name, root)?;
        let askpass = root.join("../vps-ssh/.askpass").join(format!("{name}.sh"));
        if !askpass.is_file() {
            // vps.sh writes the askpass shim on first use
            let o = site::run(Command::new(root.join("../vps-ssh/vps.sh")).args([name, "true"]))?;
            if !askpass.is_file() {
                bail!("no askpass for vps host {name}: {}", o.stderr.trim());
            }
        }
        return Ok(Dest::Vps { userip, port, askpass, base: path.into() });
    }
    if let Some((host, path)) = dest.split_once(':') {
        if host.contains('/') || path.is_empty() {
            bail!("--dest must be <dir> | <user@host>:<path> | vps:<alias>:<path>");
        }
        return Ok(Dest::Ssh { target: host.into(), base: path.into() });
    }
    Ok(Dest::Local { base: PathBuf::from(dest) })
}

impl Dest {
    /// rsync `src` (dir → trailing-slash copy) to `base/rel`, skipping
    /// any path whose top-level name is in `excludes`.
    fn push(&self, src: &Path, rel: &str, excludes: &[&str]) -> Result<()> {
        let from = if src.is_dir() {
            format!("{}/", src.display())
        } else {
            src.display().to_string()
        };
        // no --delete: deploys are additive — several pushes share one dest
        // (cache/ + www/ merge into one docroot) and remote files we did not
        // put there are never ours to remove.
        let mut c = Command::new("rsync");
        c.arg("-az").arg("--timeout=120");
        for x in excludes {
            c.arg(format!("--exclude={x}"));
        }
        match self {
            Dest::Local { base } => {
                let dst = base.join(rel);
                std::fs::create_dir_all(if src.is_dir() { dst.clone() } else { dst.parent().unwrap_or(&dst).to_path_buf() })?;
                c.arg(&from).arg(&dst);
            }
            Dest::Ssh { target, base } => {
                let dst = join_remote(base, rel);
                ssh_mkdir(target, &dst, src.is_dir())?;
                // --protect-args: the {target}:{dst} string is expanded by
                // the remote shell — send the path verbatim, no glob/subshell
                c.args(["-e", "ssh -o BatchMode=yes -o ConnectTimeout=15", "--protect-args"])
                    .arg(&from)
                    .arg(format!("{target}:{dst}"));
            }
            Dest::Vps { userip, port, askpass, base } => {
                let dst = join_remote(base, rel);
                vps_mkdir(userip, port, askpass, &dst, src.is_dir())?;
                c.args([
                    "-e",
                    &format!("ssh -p {port} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o NumberOfPasswordPrompts=1"),
                    "--protect-args",
                ])
                .env("SSH_ASKPASS", askpass)
                .env("SSH_ASKPASS_REQUIRE", "force")
                .env("DISPLAY", ":0")
                .arg(&from)
                .arg(format!("{userip}:{dst}"));
            }
        }
        let o = site::run(&mut c)?;
        if !o.ok {
            bail!("rsync {} → …/{rel} failed: {}", src.display(), o.stderr.trim());
        }
        Ok(())
    }
}

fn join_remote(base: &str, rel: &str) -> String {
    if rel.is_empty() {
        base.trim_end_matches('/').to_string()
    } else {
        format!("{}/{}", base.trim_end_matches('/'), rel)
    }
}

/// www/ entries that must never ship: `bridge/` is disposable spawn state
/// (wp-config.php with NONCE_KEY, bridge-token, .secret-*) — every target
/// either exposes www/ to the web (static/pages/vps nginx @assets) or can
/// rebuild it on first bridge request.
const WWW_EXCLUDES: &[&str] = &["bridge", ".secret-*"];

fn ssh_mkdir(target: &str, dst: &str, is_dir: bool) -> Result<()> {
    let dir = if is_dir { dst.to_string() } else { dst.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_else(|| ".".into()) };
    // --dest is user/env-controlled: the dir must be quoted before it goes
    // into the remote shell string
    let o = site::ssh(target, &format!("mkdir -p {}", site::shell_quote(&dir)))?;
    if !o.ok {
        bail!("ssh mkdir {dir}: {}", o.stderr.trim());
    }
    Ok(())
}

fn vps_mkdir(userip: &str, port: &str, askpass: &Path, dst: &str, is_dir: bool) -> Result<()> {
    let dir = if is_dir { dst.to_string() } else { dst.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_else(|| ".".into()) };
    let mut c = Command::new("ssh");
    c.args([
        "-p", port,
        "-o", "StrictHostKeyChecking=no",
        "-o", "UserKnownHostsFile=/dev/null",
        "-o", "NumberOfPasswordPrompts=1",
        userip,
        &format!("mkdir -p {}", site::shell_quote(&dir)),
    ])
    .env("SSH_ASKPASS", askpass)
    .env("SSH_ASKPASS_REQUIRE", "force")
    .env("DISPLAY", ":0");
    let o = site::run(&mut c)?;
    if !o.ok {
        bail!("ssh mkdir {dir}: {}", o.stderr.trim());
    }
    Ok(())
}

/// vps.sh alias → ("user@ip", port) read from `vps.sh list`.
fn vps_addr(name: &str, root: &Path) -> Result<(String, String)> {
    let o = site::run(Command::new(root.join("../vps-ssh/vps.sh")).arg("list"))?;
    for line in o.stdout.lines() {
        let mut it = line.split_whitespace();
        if it.next() == Some(name) {
            let addr = it.next().unwrap_or("");
            let (userip, port) = addr.rsplit_once(':').unwrap_or((addr, "22"));
            return Ok((userip.to_string(), port.to_string()));
        }
    }
    bail!("vps host {name} not in `vps.sh list`")
}

// ---------- shared steps ----------


/// Regenerate against the public domain on a COPY of the db — the sandbox
/// keeps pointing at localhost. Absolute-URL columns the renderer never
/// rewrites (menu_items.url, redirects.new_url) are rewritten here, on the
/// staged copy only: a rewrite on work/rpress.db would destroy the
/// local_url key future deploys match against.
fn stage_domain(root: &Path, work: &Path, domain: &str, env: &SiteEnv) -> Result<PathBuf> {
    let staging = work.join("deploy-cache");
    let stagedb = work.join("deploy.db");
    snapshot_db(&work.join("rpress.db"), &stagedb)?;
    {
        let db = conn(&stagedb)?;
        db.execute(sql!("site__deploy_site_url"), params![domain])?;
        rewrite_deploy_urls(&db, &env.local_url, domain)?;
    }
    generate::run(&stagedb, &root.join("templates"), &staging, domain, &env.tour_base)?;
    eprintln!("staged {} → {domain}", staging.display());
    Ok(staging)
}

/// Consistent copy of a live WAL database: `fs::copy` would lose pages
/// still in `rpress.db-wal`; `VACUUM INTO` produces a complete single-file
/// snapshot without touching the source.
fn snapshot_db(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        std::fs::remove_file(dst)?;
    }
    let c = rusqlite::Connection::open(src)?;
    c.execute("VACUUM INTO ?1", [dst.to_string_lossy().as_ref()])?;
    Ok(())
}

/// Point absolute-URL columns at the deployed domain. Keyed on every
/// origin the data can carry: env.local_url, the previously-deployed
/// domain (settings.deployed_domain, written at first deploy), and any
/// `http://localhost:*` prefix. REPLACE on the staged copy only — idempotent
/// across redeploys because each run re-keys from the settings row.
pub fn rewrite_deploy_urls(db: &rusqlite::Connection, local_url: &str, domain: &str) -> Result<()> {
    let prev: String = db
        .query_row(
            sql!("site_deploy__select_deployed_domain"),
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let mut origins: Vec<String> = Vec::new();
    for o in [local_url.trim_end_matches('/'), prev.trim_end_matches('/'), domain.trim_end_matches('/')] {
        if !o.is_empty() && o != domain && !origins.iter().any(|x| x == o) {
            origins.push(o.to_string());
        }
    }
    for origin in &origins {
        for (table, field) in [("menu_items", "url"), ("redirects", "new_url")] {
            db.execute(
                &format!("UPDATE {table} SET {field}=REPLACE({field}, ?1, ?2) WHERE {field} LIKE ?1 || '%'"),
                params![origin, domain],
            )?;
        }
    }
    // `http://localhost:PORT/path` — port varies, so rewrite the scheme+
    // host+port prefix and keep the path suffix.
    for (table, field) in [("menu_items", "url"), ("redirects", "new_url")] {
        db.execute(
            &format!(
                "UPDATE {table} SET {field} = ?1 || SUBSTR({field}, 17 + INSTR(SUBSTR({field}, 18), '/')) \
                 WHERE {field} LIKE 'http://localhost:%' AND INSTR(SUBSTR({field}, 18), '/') > 0"
            ),
            params![domain],
        )?;
    }
    db.execute(
        sql!("site_deploy__upsert_deployed_domain"),
        params![domain],
    )?;
    Ok(())
}

fn need_bin(name: &str, hint: &str) -> Result<()> {
    let o = site::run(Command::new("sh").args(["-c", &format!("command -v {name}")]))?;
    if !o.ok || o.stdout.trim().is_empty() {
        bail!("missing `{name}` — {hint}");
    }
    Ok(())
}

/// Record the last deployed domain on the WORK db after a successful
/// deploy. The staged copy that carried the row is discarded; without this
/// the next deploy reads deployed_domain='' and can't re-key rows that
/// were already pointing at the previous domain. Bookkeeping only —
/// URL columns in the work db are never touched (they keep local_url so
/// future staged rewrites always have their match key).
pub fn mark_deployed(work_db: &Path, domain: &str) -> Result<()> {
    let db = conn(work_db)?;
    db.execute(
        sql!("site_deploy__upsert_deployed_domain"),
        params![domain],
    )?;
    Ok(())
}

/// `cp -a` equivalent without non-portable flags. `skip` matches top-level
/// entry names per recursion level (`.secret-*` style: trailing `*` = prefix).
fn copy_tree(src: &Path, dst: &Path, skip: &[&str]) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for e in std::fs::read_dir(src)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().to_string();
        if skip.iter().any(|p| match p.strip_suffix('*') {
            Some(pre) => name.starts_with(pre),
            None => name == *p,
        }) {
            continue;
        }
        let to = dst.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_tree(&e.path(), &to, skip)?;
        } else {
            std::fs::copy(e.path(), &to)?;
        }
    }
    Ok(())
}

// ---------- targets ----------

fn deploy_static(a: &DeployArgs, root: &Path, env: &SiteEnv, work: &Path, domain: &str) -> Result<()> {
    let dest = a.dest.clone().context("static deploy needs --dest <dir|user@host:/path|vps:alias:/path>")?;
    if a.dry_run {
        eprintln!("dry-run: regenerate with site_url={domain}, then rsync cache/ + www/ → {dest}");
        return Ok(());
    }
    let dest = parse_dest(&dest, root)?;
    let staging = stage_domain(root, work, domain, env)?;
    dest.push(&staging, "", &[])?;
    dest.push(&work.join("www"), "", WWW_EXCLUDES)?;
    mark_deployed(&work.join("rpress.db"), domain)?;
    eprintln!("deployed static {} → {domain}", env.site);
    eprintln!("note: vhost docroot = the synced dir; dynamic routes are off");
    Ok(())
}

fn deploy_pages(a: &DeployArgs, root: &Path, env: &SiteEnv, work: &Path, domain: &str) -> Result<()> {
    need_bin("wrangler", "npm i -g wrangler")?;
    let project = a.project.clone().unwrap_or_else(|| env.site.clone());
    if a.dry_run {
        eprintln!("dry-run: regenerate with {domain}, merge cache+www → work/{}/deploy-site, `wrangler pages deploy --project-name {project}`", env.site);
        return Ok(());
    }
    let staging = stage_domain(root, work, domain, env)?;
    let site_dir = work.join("deploy-site");
    if site_dir.exists() {
        std::fs::remove_dir_all(&site_dir)?;
    }
    copy_tree(&staging, &site_dir, &[])?;
    copy_tree(&work.join("www"), &site_dir, WWW_EXCLUDES)?;
    let o = site::run(Command::new("wrangler").args([
        "pages", "deploy",
        &site_dir.display().to_string(),
        "--project-name", &project,
    ]))?;
    if !o.ok {
        bail!("wrangler deploy failed: {}", o.stderr.trim());
    }
    mark_deployed(&work.join("rpress.db"), domain)?;
    eprintln!("deployed {} → Cloudflare Pages project {project}", env.site);
    eprintln!("note: bind {domain} as a custom domain on the project if not already");
    Ok(())
}

fn deploy_vps(a: &DeployArgs, root: &Path, env: &SiteEnv, work: &Path, domain: &str) -> Result<()> {
    let dest = a.dest.clone().context("vps deploy needs --dest <user@host:/appdir|vps:alias:/appdir>")?;
    let bin = root.join("target/release/rpress");
    if !bin.is_file() {
        bail!("no release binary — run `cargo build --release` first");
    }
    // The pushed binary must be a Linux ELF — building on macOS leaves a
    // Mach-O that fails at ExecStart with a cryptic error on the remote.
    // Reject unless --yes/--dry-run already signaled intent? No: this is a
    // hard mismatch, not a preference — bail always.
    {
        let mut magic = [0u8; 4];
        let mut f = std::fs::File::open(&bin)?;
        use std::io::Read;
        let _ = f.read(&mut magic)?;
        anyhow::ensure!(
            magic == [0x7F, b'E', b'L', b'F'],
            "target/release/rpress is not a Linux ELF (built on this host's OS) — cross-compile for linux (e.g. `cargo build --release --target x86_64-unknown-linux-gnu` or build on the VPS) before `site deploy --target vps`"
        );
    }
    if a.dry_run {
        eprintln!("dry-run: rsync binary+db+templates+www → {dest}, then print nginx+systemd runbook");
        return Ok(());
    }
    let dest = parse_dest(&dest, root)?;
    // payload: binary + rewritten db + templates + www; cache/ regenerates
    // on the host. The pushed db is a copy rewritten to the domain — the
    // sandbox keeps local_url (menu_items.url/redirects.new_url would
    // otherwise ship localhost links).
    let stagedb = work.join("deploy.db");
    snapshot_db(&work.join("rpress.db"), &stagedb)?;
    {
        let db = conn(&stagedb)?;
        db.execute(sql!("site__deploy_site_url"), params![domain])?;
        rewrite_deploy_urls(&db, &env.local_url, domain)?;
    }
    dest.push(&bin, "bin/rpress", &[])?;
    dest.push(&stagedb, "rpress.db", &[])?;
    dest.push(&root.join("templates"), "templates", &[])?;
    dest.push(&work.join("www"), "www", WWW_EXCLUDES)?;
    eprintln!("app payload pushed — remote needs a first `generate` + the runbook below");
    mark_deployed(&work.join("rpress.db"), domain)?;
    print_runbook(env, domain, a.app_dir.as_deref());
    Ok(())
}

fn print_runbook(env: &SiteEnv, domain: &str, app_dir: Option<&str>) {
    let app_dir = app_dir.unwrap_or("<app-dir>");
    eprintln!(
        r#"
== runbook: {domain} ==

1) first generate on the host (produces cache/ + pagefind index)

   {app_dir}/bin/rpress generate \
     --db {app_dir}/rpress.db \
     --templates {app_dir}/templates \
     --out {app_dir}/cache --site-url {domain}

2) systemd unit — /etc/systemd/system/rpress-{site}.service

   [Service]
   ExecStart={app_dir}/bin/rpress serve \
     --bind 127.0.0.1:8400 \
     --db {app_dir}/rpress.db \
     --templates {app_dir}/templates \
     --out {app_dir}/cache \
     --www {app_dir}/www \
     --host {domain} --site {site} --name "{name}" --tour-base {tour_base}
   Restart=always
   # Confinement — covers rpress AND any bridged php -S child (Linux has
   # no sandbox-exec; this IS the sandbox layer there):
   ProtectSystem=strict
   ProtectHome=yes
   PrivateTmp=yes
   NoNewPrivileges=yes
   ReadWritePaths={app_dir}
   # Bridge children must never reach the network beyond loopback:
   RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
3) nginx — dynamic routes to the process, static straight from disk

   location ~ ^/(admin|api|comment|review|form|intake) {{ proxy_pass http://127.0.0.1:8400; }}
   location / {{ root {app_dir}/cache; try_files $uri $uri/ @assets; }}
   location @assets {{ root {app_dir}/www; }}

4) systemctl enable --now rpress-{site}
"#,
        site = env.site,
        name = env.name,
        tour_base = env.tour_base,
    );
}

/// `https?://host[:port]` — host chars only, no path/query/creds.
/// Deliberately small: deploy writes this verbatim into site_url.
pub(crate) fn valid_origin(s: &str) -> bool {
    let rest = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
        .unwrap_or("");
    if rest.is_empty() || rest.contains(['/', '?', '#', '@', ' ']) {
        return false;
    }
    let (host, port) = match rest.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => (h, Some(p)),
        Some(_) => return false,
        None => (rest, None),
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty()
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
    {
        return false;
    }
    if let Some(p) = port {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::valid_origin;

    #[test]
    fn origins() {
        for ok in [
            "https://example.com",
            "http://example.com",
            "https://sub.example.co.uk",
            "https://example.com:8443",
            "http://127.0.0.1:8000",
        ] {
            assert!(valid_origin(ok), "{ok}");
        }
        for bad in [
            "",
            "http",
            "httpfoo",
            "https://",
            "https://example.com/",
            "https://example.com/path",
            "https://user@example.com",
            "https://example.com:abc",
            "https://example.com:99999",
            "ftp://example.com",
            "https://exa mple.com",
        ] {
            assert!(!valid_origin(bad), "{bad}");
        }
    }
}
