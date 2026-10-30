//! `rpress bridge status|kill` — operator debug surface for PhpBridge
//! runtimes (phase-06 T23/T26). Live bridges live inside `rpress serve`;
//! this tool is *disk-level*: it inspects and removes each plugin's
//! `www/bridge/<slug>` workdir (the disposable wp.db lives there).
//!
//! `status` lists workdirs (slug, wp.db size, last-write age).
//! `kill` removes the workdir — a running bridge keeps answering until
//! the idle TTL reaps it, but its wp.db is gone, so the next spawn is
//! a clean rebuild (bounded blast radius by design; wp.db is never a
//! source of truth).

use crate::site;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::time::SystemTime;

pub struct BridgeArgs {
    /// Site slug (sites/<slug>.env) — locates the site's www dir.
    pub site: String,
    pub action: BridgeAction,
}

#[derive(clap::Subcommand)]
pub enum BridgeAction {
    /// List bridge workdirs (wp.db size + last-write age)
    Status,
    /// Remove a plugin's bridge workdir — forces clean rebuild on next request
    Kill {
        /// Plugin slug, e.g. goodlayers
        slug: String,
    },
}

pub fn run(a: BridgeArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let www = work_www(&root, &a.site)?;
    let base = www.join("bridge");
    match &a.action {
        BridgeAction::Status => status(&base),
        BridgeAction::Kill { slug } => kill(&base, slug),
    }
}

fn work_www(root: &Path, site: &str) -> Result<std::path::PathBuf> {
    // Same layout site_build uses: work/<site>/{cache,www,rpress.db}.
    let w = root.join("work").join(site).join("www");
    anyhow::ensure!(w.is_dir(), "no {} — run `rpress site build {site}` first", w.display());
    Ok(w)
}

fn status(base: &Path) -> Result<()> {
    if !base.is_dir() {
        println!("no bridge workdirs ({})", base.display());
        return Ok(());
    }
    println!("{:<24} {:>12} {:>10}", "slug", "wp.db bytes", "age");
    for e in std::fs::read_dir(base).context("read bridge dir")? {
        let e = e?;
        let dir = e.path();
        if !dir.is_dir() {
            continue;
        }
        let slug = e.file_name().to_string_lossy().into_owned();
        let db = dir.join("wp.db");
        let (size, age) = match std::fs::metadata(&db) {
            Ok(m) => {
                let age = m
                    .modified()
                    .ok()
                    .and_then(|t| SystemTime::now().duration_since(t).ok())
                    .map(|d| format!("{}s", d.as_secs()))
                    .unwrap_or_else(|| "?".into());
                (m.len(), age)
            }
            Err(_) => (0, "-".into()),
        };
        println!("{slug:<24} {size:>12} {age:>10}");
    }
    Ok(())
}

fn kill(base: &Path, slug: &str) -> Result<()> {
    // slug → remove_dir_all: a path like "../.." must never leave www/bridge.
    if !site::valid_slug(slug) {
        bail!("invalid slug {slug:?} — want [a-z0-9][a-z0-9-]*");
    }
    let dir = base.join(slug);
    if dir.is_dir() {
        std::fs::remove_dir_all(&dir).with_context(|| format!("remove {}", dir.display()))?;
        println!("removed {} — next request rebuilds wp.db fresh", dir.display());
    } else {
        println!("no workdir for slug {slug:?} — nothing to do");
    }
    Ok(())
}
