//! `rpress site build` — the e2e pipeline as native phases.
//!
//!   pull → import → configure → generate → check
//!
//! Every phase reports OK/WARN/FAIL with a hint on failure, so a broken run
//! says *which* part broke and what to do next — no digging through a log.
//! `--json` prints a machine-readable report on stdout for agents.

use crate::db::conn;
use crate::site::{self, Remote, SiteEnv};
use crate::{auth, generate, import, verify};
// lib.rs brings sql! in via #[macro_use]; the bin target does not — this
// import is required there and merely redundant here.
#[allow(unused_imports)]
use crate::sql;
use anyhow::{bail, Context, Result};
use rusqlite::params;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub struct BuildArgs {
    pub site: String,
    /// comma list, e.g. "generate,check" — default: all phases
    pub only: Option<String>,
    /// resume from this phase onward
    pub from: Option<String>,
    pub json: bool,
    pub templates: Option<PathBuf>,
}

struct Ctx {
    root: PathBuf,
    env: SiteEnv,
    work: PathBuf,
    db: PathBuf,
    cache: PathBuf,
    www: PathBuf,
    dump: PathBuf,
    templates: PathBuf,
}

struct PhaseOut {
    detail: String,
    warns: Vec<String>,
}

#[derive(serde::Serialize)]
struct Report {
    site: String,
    phases: Vec<PhaseReport>,
}

#[derive(serde::Serialize)]
struct PhaseReport {
    phase: &'static str,
    status: &'static str, // ok | warn | fail | skip
    detail: String,
    secs: f64,
}

const PHASES: [(&str, fn(&Ctx) -> Result<PhaseOut>); 5] = [
    ("pull", pull),
    ("import", import_phase),
    ("configure", configure),
    ("generate", generate_phase),
    ("check", check),
];

pub fn run(a: BuildArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let env = SiteEnv::load(&root, &a.site)?;
    let work = root.join("work").join(&env.site);
    let ctx = Ctx {
        db: work.join("rpress.db"),
        cache: work.join("cache"),
        www: work.join("www"),
        dump: work.join("dump.sql"),
        templates: a.templates.unwrap_or_else(|| root.join("templates")),
        work,
        env,
        root,
    };

    let names: Vec<&str> = PHASES.iter().map(|(n, _)| *n).collect();
    let from = a.from.as_deref().unwrap_or("");
    if !from.is_empty() && !names.contains(&from) {
        bail!("unknown phase {from:?} — valid: {}", names.join(","));
    }
    let only: Vec<&str> = a
        .only
        .as_deref()
        .map(|s| s.split(',').map(str::trim).collect())
        .unwrap_or_default();
    for o in &only {
        if !names.contains(o) {
            bail!("unknown phase {o:?} — valid: {}", names.join(","));
        }
    }
    let mut started = from.is_empty();
    let mut report = Report { site: ctx.env.site.clone(), phases: vec![] };

    for (name, f) in PHASES {
        if !started {
            started = name == from;
            if !started {
                report.phases.push(skip(name, "before --from"));
                continue;
            }
        }
        if !only.is_empty() && !only.contains(&name) {
            report.phases.push(skip(name, "not in --only"));
            continue;
        }
        if !a.json {
            eprintln!("\n=== [{name}] ===");
        }
        let t = Instant::now();
        match f(&ctx) {
            Ok(out) => {
                let status = if out.warns.is_empty() { "ok" } else { "warn" };
                for w in &out.warns {
                    eprintln!("WARN {w}");
                }
                eprintln!("[{name}] {status}: {} ({:.0?})", out.detail, t.elapsed());
                report.phases.push(PhaseReport {
                    phase: name,
                    status,
                    detail: out.detail,
                    secs: t.elapsed().as_secs_f64(),
                });
            }
            Err(e) => {
                eprintln!("[{name}] FAIL: {e:#} ({:.0?})", t.elapsed());
                report.phases.push(PhaseReport {
                    phase: name,
                    status: "fail",
                    detail: format!("{e:#}"),
                    secs: t.elapsed().as_secs_f64(),
                });
                if a.json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                bail!("phase {name} failed — fix above, then resume: rpress site build {} --from {name}", ctx.env.site);
            }
        }
    }
    if a.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        eprintln!(
            "\n== done ==  next: rpress site serve {}",
            ctx.env.site
        );
    }
    Ok(())
}

fn skip(phase: &'static str, why: &str) -> PhaseReport {
    PhaseReport { phase, status: "skip", detail: why.into(), secs: 0.0 }
}

fn out(detail: impl Into<String>) -> Result<PhaseOut> {
    Ok(PhaseOut { detail: detail.into(), warns: vec![] })
}

fn warn_out(detail: impl Into<String>, warns: Vec<String>) -> Result<PhaseOut> {
    Ok(PhaseOut { detail: detail.into(), warns })
}

// ---------- pull ----------

fn pull(c: &Ctx) -> Result<PhaseOut> {
    if !c.env.clone_source_ok {
        bail!("CLONE_SOURCE_OK is not yes in sites/{}.env — pull only reads clones, never prod", c.env.site);
    }
    if let Err(e) = need_bin("rsync") {
        eprintln!("[pull] {e} — remote pulls will use the tar/base64 stream transport");
    }
    std::fs::create_dir_all(&c.work)?;

    // dump
    if let Some(local) = &c.env.dump_local {
        std::fs::copy(local, &c.dump)
            .with_context(|| format!("copy {} → {}", local.display(), c.dump.display()))?;
    } else if let Some(remote_path) = &c.env.dump_remote {
        remote_pull(c, remote_path, &c.work.join("dump-src"))?;
        let pulled = c.work.join("dump-src").join(
            Path::new(remote_path).file_name().unwrap_or_default(),
        );
        std::fs::rename(&pulled, &c.dump)
            .with_context(|| format!("remote dump missing after pull: {}", pulled.display()))?;
        let _ = std::fs::remove_dir(c.work.join("dump-src"));
    } else {
        bail!("env has neither DUMP_LOCAL nor DUMP_REMOTE — re-run `rpress site init`");
    }
    if !c.dump.is_file() || std::fs::metadata(&c.dump)?.len() == 0 {
        bail!("{} is empty/missing after pull", c.dump.display());
    }

    // wp-content + wp-includes assets
    let mut pulled = vec![];
    if let Some(local_wc) = &c.env.wp_content_local {
        for part in ["uploads", "themes", "plugins"] {
            let src = local_wc.join(part);
            if src.is_dir() {
                rsync_local(&src, &c.www.join("wp-content").join(part))?;
                pulled.push(format!("wp-content/{part}"));
            }
        }
        let inc = local_wc.parent().unwrap_or(local_wc).join("wp-includes");
        for part in ["js", "css", "fonts", "images"] {
            let src = inc.join(part);
            if src.is_dir() {
                rsync_local(&src, &c.www.join("wp-includes").join(part))?;
                pulled.push(format!("wp-includes/{part}"));
            }
        }
    } else if let Some(wp_root) = &c.env.wp_root {
        for part in ["uploads", "themes", "plugins"] {
            remote_pull(c, &format!("{wp_root}/wp-content/{part}/"), &c.www.join("wp-content").join(part))?;
            pulled.push(format!("wp-content/{part}"));
        }
        for part in ["js", "css", "fonts", "images"] {
            remote_pull(c, &format!("{wp_root}/wp-includes/{part}/"), &c.www.join("wp-includes").join(part))?;
            pulled.push(format!("wp-includes/{part}"));
        }
    } else {
        eprintln!("WARN no wp-content source — pages render but assets 307 to origin");
    }

    // repo-owned front assets — always local, never a 307
    std::fs::create_dir_all(&c.www)?;
    for f in ["front.css", "nav.js"] {
        let src = c.root.join("www").join(f);
        if src.is_file() {
            std::fs::copy(&src, c.www.join(f))?;
        }
    }

    let size = std::fs::metadata(&c.dump)?.len() / (1 << 20);
    out(format!("{size}MB dump, {} asset trees", pulled.len()))
}

/// rsync a local dir into dest (mkdir -p first, trailing-slash semantics).
fn rsync_local(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let o = site::run(Command::new("rsync").args([
        "-a",
        &format!("{}/", src.display()),
        &format!("{}/", dest.display()),
    ]))?;
    if !o.ok {
        bail!("rsync {} failed: {}", src.display(), o.stderr.trim());
    }
    Ok(())
}

/// Pull a remote path into `dest`.
///
/// Transports, in order: rsync (fast, resumable) → tar+base64 stream over a
/// plain ssh channel. The stream fallback exists because real VPSes break
/// rsync in several ways — missing/old remote rsync, banner text corrupting
/// the protocol, restricted shells that only allow plain ssh commands —
/// and a bare "unexpected end of file" tells the user nothing. Preflight
/// checks report *which* of those it actually is.
fn remote_pull(c: &Ctx, remote_path: &str, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    let remote = c.remote()?;
    let label = remote.label();

    remote.lines("echo rpress-ok").map_err(|e| {
        anyhow::anyhow!("cannot reach {label} — check ssh key/agent or VPN: {e:#}")
    })?;

    let q = shell_quote(remote_path);
    let kind = remote
        .lines(&format!(
            "if test -d {q}; then echo dir; elif test -f {q}; then echo file; fi"
        ))?
        .into_iter()
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "remote path missing: {remote_path} — re-check WP_ROOT / DUMP_REMOTE on {label}"
            )
        })?;

    match rsync_pull(&remote, remote_path, dest) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!(
                "[pull] rsync {label}:{remote_path} failed ({}) — falling back to tar/base64 stream",
                one_line(&e.to_string())
            );
            stream_pull(&remote, remote_path, dest, kind == "dir")
        }
    }
}

/// rsync a remote path — vps.sh askpass helper or plain ssh BatchMode.
fn rsync_pull(remote: &Remote, remote_path: &str, dest: &Path) -> Result<()> {
    match remote {
        Remote::Vps { name, script } => {
            let pull = script
                .parent()
                .unwrap_or(Path::new("."))
                .join(".askpass/rsync-pull.sh");
            let o = site::run(Command::new(&pull).args([
                name,
                remote_path,
                &format!("{}/", dest.display()),
            ]))?;
            if !o.ok {
                bail!("{}", o.stderr.trim());
            }
        }
        Remote::Ssh { target } => {
            let o = site::run(Command::new("rsync").args([
                "-az",
                "--timeout=120",
                "-e",
                "ssh -o BatchMode=yes -o ConnectTimeout=15",
                &format!("{target}:{remote_path}"),
                &format!("{}/", dest.display()),
            ]))?;
            if !o.ok {
                bail!("{}", o.stderr.trim());
            }
        }
    }
    Ok(())
}

/// Last-resort transport for hosts where rsync/scp fail (protocol mismatch,
/// banner-mangled streams, restricted shells): tar+base64 through the same
/// plain ssh channel `Remote::exec` already proves works. Slower and not
/// resumable, but it survives broken remote rsyncs and mangled pipes.
fn stream_pull(remote: &Remote, remote_path: &str, dest: &Path, is_dir: bool) -> Result<()> {
    need_bin("base64")?;
    let q = shell_quote(remote_path);
    let (remote_cmd, local_sh) = if is_dir {
        need_bin("tar")?;
        (
            format!("cd {q} && tar czf - . | base64"),
            format!("base64 -d | tar xzf - -C {}", shell_quote(&dest.to_string_lossy())),
        )
    } else {
        let name = Path::new(remote_path).file_name().unwrap_or_default();
        (
            format!("base64 {q}"),
            format!(
                "base64 -d > {}",
                shell_quote(&dest.join(name).to_string_lossy())
            ),
        )
    };
    stream_pipeline(remote, &remote_cmd, &local_sh)
        .with_context(|| format!("stream pull {remote_path}"))
}

/// Pipe a remote command's stdout into a local shell pipeline without
/// buffering the payload in memory.
fn stream_pipeline(remote: &Remote, remote_cmd: &str, local_sh: &str) -> Result<()> {
    let mut prod = match remote {
        Remote::Vps { name, script } => Command::new(script)
            .args([name.as_str(), remote_cmd])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn(),
        Remote::Ssh { target } => Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=15",
                target.as_str(),
                remote_cmd,
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn(),
    }
    .context("spawn remote producer")?;

    // drain remote stderr on a side thread so a chatty remote can't
    // deadlock the pipe
    let err_thr = prod.stderr.take().map(|mut e| {
        std::thread::spawn(move || {
            let mut s = String::new();
            let _ = std::io::Read::read_to_string(&mut e, &mut s);
            s
        })
    });

    let mut cons = Command::new("sh")
        .args(["-c", local_sh])
        .stdin(prod.stdout.take().context("producer stdout")?)
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("spawn local consumer")?;

    let pst = prod.wait()?;
    let cst = cons.wait()?;
    let perr = err_thr
        .and_then(|t| t.join().ok())
        .unwrap_or_default();
    if !pst.success() {
        bail!("remote stream failed: {}", perr.trim());
    }
    if !cst.success() {
        bail!("local unpack failed: {local_sh}");
    }
    Ok(())
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn one_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

fn need_bin(name: &str) -> Result<()> {
    let o = site::run(Command::new("sh").args(["-c", &format!("command -v {name}")]))?;
    if !o.ok || o.stdout.trim().is_empty() {
        bail!("missing `{name}` on PATH");
    }
    Ok(())
}

impl Ctx {
    fn remote(&self) -> Result<Remote> {
        if let Some(v) = &self.env.vps_host {
            return Ok(Remote::Vps {
                name: v.clone(),
                script: std::env::var("VPS_SSH")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| self.root.join("../vps-ssh/vps.sh")),
            });
        }
        if let Some(t) = &self.env.ssh_target {
            return Ok(Remote::Ssh { target: t.clone() });
        }
        bail!("site has no remote source")
    }
}

// ---------- import ----------

fn import_phase(c: &Ctx) -> Result<PhaseOut> {
    for f in [&c.db, &c.db.with_extension("db-shm"), &c.db.with_extension("db-wal")] {
        let _ = std::fs::remove_file(f);
    }
    let _ = std::fs::remove_dir_all(&c.cache);
    import::run(&c.dump, &c.db, &c.work.join("import-report.txt"))?;
    verify::run(&c.db)?;
    let db_mb = std::fs::metadata(&c.db)?.len() / (1 << 20);
    out(format!("rpress.db {db_mb}MB, report → work/{}/import-report.txt", c.env.site))
}

// ---------- configure ----------

fn configure(c: &Ctx) -> Result<PhaseOut> {
    let db = conn(&c.db)?;
    let old_url: Option<String> = db
        .query_row(sql!("render__select_value_from_settings"), params!["site_url"], |r| r.get(0))
        .ok();
    db.execute(
        sql!("site__configure_settings"),
        params![c.env.local_url, old_url.unwrap_or_default()],
    )?;
    let legacy: String = db
        .query_row(sql!("render__select_value_from_settings"), params!["legacy_origin"], |r| r.get(0))?;
    let menus = db.execute(
        sql!("site__rewrite_menu_urls"),
        params![legacy, c.env.local_url],
    )?;
    let redirects = db.execute(
        sql!("site__rewrite_redirect_urls"),
        params![legacy, c.env.local_url],
    )?;

    // first admin — idempotent: only when users table is empty
    let users: i64 = db.query_row(sql!("site__count_users"), [], |r| r.get(0))?;
    let mut detail = format!(
        "site_url→{}, {menus} menu urls, {redirects} redirect urls",
        c.env.local_url
    );
    if users == 0 {
        let pass: String = format!("{:x}", rand::random::<u128>());
        let pass = &pass[..20];
        let email = format!("admin@{}.local", c.env.site);
        auth::add_user(&c.db, &email, pass, "admin")?;
        let creds = c.work.join("credentials.txt");
        std::fs::write(&creds, format!("email={email}\npassword={pass}\n"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&creds, std::fs::Permissions::from_mode(0o600))?;
        }
        detail += &format!("; admin → {}", creds.display());
    }
    out(detail)
}

// ---------- generate ----------

fn generate_phase(c: &Ctx) -> Result<PhaseOut> {
    generate::run(&c.db, &c.templates, &c.cache, &c.env.local_url)?;
    let llms = c.cache.join("llms.txt");
    if llms.is_file() {
        std::fs::copy(&llms, c.www.join("llms.txt"))?;
    }
    out(format!("cache → {}", c.cache.display()))
}

// ---------- check ----------

fn check(c: &Ctx) -> Result<PhaseOut> {
    let db = conn(&c.db)?;
    let front: i64 = db
        .query_row(sql!("settings__select_int"), params!["page_on_front"], |r| r.get(0))
        .unwrap_or(0);
    let primary: String = db
        .query_row(sql!("render__select_value_from_settings"), params!["primary_lang"], |r| r.get(0))
        .unwrap_or_else(|_| "fr".into());
    let langs: String = db
        .query_row(sql!("render__select_value_from_settings"), params!["languages"], |r| r.get(0))
        .unwrap_or_else(|_| primary.clone());

    let mut stmt = db.prepare(sql!("site__sample_pages"))?;
    let rows: Vec<(i64, String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
        .collect::<rusqlite::Result<_>>()?;
    drop(stmt);

    let mut warns = vec![];
    let mut misses = vec![];
    for (id, slug, lang, kind) in &rows {
        let mut rel = slug.clone();
        if kind == "tour" {
            rel = format!("{}/{rel}", c.env.tour_base);
        }
        if lang != &primary {
            rel = format!("{lang}/{rel}");
        }
        if *id == front {
            rel.clear();
        }
        let f = c.cache.join(&rel).join("index.html");
        if f.is_file() {
            eprintln!("ok   /{rel}");
        } else {
            eprintln!("MISS /{rel}");
            misses.push(format!("/{rel}"));
        }
    }
    for u in ["index.html", "sitemap.xml", "robots.txt"] {
        if !c.cache.join(u).is_file() {
            warns.push(format!("cache/{u} missing"));
        }
    }
    for lang in langs.split(',') {
        if lang == primary {
            continue;
        }
        if !c.cache.join(lang).join("index.html").is_file() {
            warns.push(format!("cache/{lang}/index.html missing"));
        }
    }
    if !c.www.join("llms.txt").is_file() {
        warns.push("www/llms.txt missing".into());
    }

    let origin: String = db
        .query_row(sql!("render__select_value_from_settings"), params!["asset_origin"], |r| r.get(0))
        .unwrap_or_default();
    if !origin.is_empty() && origin != c.env.local_url {
        let o = site::run(Command::new("sh").args([
            "-c",
            &format!(
                "grep -rl '{}/' '{}' --include='*.html' 2>/dev/null | head -3 | wc -l",
                origin,
                c.cache.display()
            ),
        ]))?;
        let n: usize = o.stdout.trim().parse().unwrap_or(0);
        if n > 0 {
            warns.push(format!("{n}+ pages still link {origin} (content-level refs)"));
        }
    }
    if !misses.is_empty() {
        bail!("{} missing pages: {}", misses.len(), misses.join(" "));
    }
    if site::run(Command::new("sh").args(["-c", "command -v pagefind"])).map(|o| o.ok).unwrap_or(false)
        && !c.cache.join("pagefind/pagefind.js").is_file()
    {
        warns.push("pagefind installed but no index — rerun generate".into());
    }
    warn_out(format!("{} sampled pages, langs={langs}", rows.len()), warns)
}
