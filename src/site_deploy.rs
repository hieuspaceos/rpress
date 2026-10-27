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
    if !domain.starts_with("http") {
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
    /// rsync `src` (dir → trailing-slash copy) to `base/rel`.
    fn push(&self, src: &Path, rel: &str) -> Result<()> {
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
/// keeps pointing at localhost.
fn stage_domain(root: &Path, work: &Path, domain: &str) -> Result<PathBuf> {
    let staging = work.join("deploy-cache");
    let stagedb = work.join("deploy.db");
    std::fs::copy(work.join("rpress.db"), &stagedb)?;
    {
        let db = conn(&stagedb)?;
        db.execute(sql!("site__deploy_site_url"), params![domain])?;
    }
    generate::run(&stagedb, &root.join("templates"), &staging, domain)?;
    eprintln!("staged {} → {domain}", staging.display());
    Ok(staging)
}

fn need_bin(name: &str, hint: &str) -> Result<()> {
    let o = site::run(Command::new("sh").args(["-c", &format!("command -v {name}")]))?;
    if !o.ok || o.stdout.trim().is_empty() {
        bail!("missing `{name}` — {hint}");
    }
    Ok(())
}

/// `cp -a` equivalent without non-portable flags.
fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for e in std::fs::read_dir(src)? {
        let e = e?;
        let to = dst.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_tree(&e.path(), &to)?;
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
    let staging = stage_domain(root, work, domain)?;
    dest.push(&staging, "")?;
    dest.push(&work.join("www"), "")?;
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
    let staging = stage_domain(root, work, domain)?;
    let site_dir = work.join("deploy-site");
    if site_dir.exists() {
        std::fs::remove_dir_all(&site_dir)?;
    }
    copy_tree(&staging, &site_dir)?;
    copy_tree(&work.join("www"), &site_dir)?;
    let o = site::run(Command::new("wrangler").args([
        "pages", "deploy",
        &site_dir.display().to_string(),
        "--project-name", &project,
    ]))?;
    if !o.ok {
        bail!("wrangler deploy failed: {}", o.stderr.trim());
    }
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
    if a.dry_run {
        eprintln!("dry-run: rsync binary+db+templates+www → {dest}, then print nginx+systemd runbook");
        return Ok(());
    }
    let dest = parse_dest(&dest, root)?;
    // payload: binary + db + templates + www; cache/ regenerates on the host
    dest.push(&bin, "bin/rpress")?;
    dest.push(&work.join("rpress.db"), "rpress.db")?;
    dest.push(&root.join("templates"), "templates")?;
    dest.push(&work.join("www"), "www")?;
    eprintln!("app payload pushed — remote needs a first `generate` + the runbook below");
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
