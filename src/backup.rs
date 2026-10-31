//! Site backup — one tar.gz per snapshot under `work/<site>/backups/`.
//! Contents: `rpress.db` (consistent `VACUUM INTO` snapshot — never a
//! raw copy of a live DB), `www/` (uploads + vendored assets) and the
//! site's `sites/<site>.env`. Rotation keeps the newest `keep` archives.
//!
//! `rpress site init --backup <archive>` is the documented restore path;
//! there is no restore UI by design.

use anyhow::{bail, Context, Result};
use crate::sql;
use std::path::{Path, PathBuf};

/// Create a backup archive for the site whose workdir is `work/<site>`.
/// Returns the archive path. `env_file` may be empty → env skipped.
pub fn run_backup(work: &Path, env_file: &Path, keep: usize) -> Result<PathBuf> {
    run_backup_db(work, &work.join("rpress.db"), env_file, keep)
}

/// `run_backup` with the DB path explicit — `serve --db` accepts any
/// filename, so callers that know the real path pass it instead of
/// assuming `rpress.db`.
pub fn run_backup_db(work: &Path, db: &Path, env_file: &Path, keep: usize) -> Result<PathBuf> {
    let keep = keep.max(1); // keep=0 would delete the archive just made
    anyhow::ensure!(db.is_file(), "no database at {}", db.display());
    let dir = work.join("backups");
    std::fs::create_dir_all(&dir)?;

    // Consistent snapshot via VACUUM INTO (WAL-safe, no live-file copy).
    // VACUUM INTO refuses an existing target, so a .snapshot.db left behind
    // by a crashed run would brick every later backup — remove it first.
    let snap = dir.join(".snapshot.db");
    if snap.exists() {
        std::fs::remove_file(&snap)
            .with_context(|| format!("remove stale snapshot {}", snap.display()))?;
    }
    {
        let c = rusqlite::Connection::open(&db)?;
        c.execute(sql!("backup__vacuum_into"), [snap.to_string_lossy().as_ref()])?;
    }

    // Stage with symlinks + `tar -h` (dereference) — archives
    // multi-GB www/ without copying, works on bsdtar (no --transform).
    let stage = dir.join(".stage");
    if stage.exists() {
        std::fs::remove_dir_all(&stage)?;
    }
    std::fs::create_dir_all(&stage)?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&snap, stage.join("rpress.db"))?;
        if work.join("www").is_dir() {
            std::os::unix::fs::symlink(work.join("www"), stage.join("www"))?;
        }
        if env_file.is_file() {
            std::os::unix::fs::symlink(env_file, stage.join("site.env"))?;
        }
    }
    let name = format!("{}.tar.gz", chrono_now());
    let archive = dir.join(&name);
    let out = std::process::Command::new("tar")
        .args(["-chzf"]).arg(&archive)
        // www/bridge/ is disposable spawn state (wp-config.php with nonce
        // salts, bridge-token, .secret-*) — never a backup artifact, and
        // dereferencing it would embed a whole WP tree per archive.
        .arg("--exclude=./www/bridge")
        .arg("-C").arg(&stage)
        .arg(".")
        .output().context("spawn tar")?;
    std::fs::remove_dir_all(&stage).ok();
    std::fs::remove_file(&snap).ok();
    anyhow::ensure!(out.status.success(), "tar failed: {}", String::from_utf8_lossy(&out.stderr));

    rotate(&dir, keep)?;
    Ok(archive)
}

fn chrono_now() -> String {
    // No chrono dep — timestamp via `date`.
    let o = std::process::Command::new("date")
        .args(["+%Y%m%d-%H%M%S"])
        .output();
    match o {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
    }
}


/// Keep the newest `keep` `*.tar.gz`; delete the rest.
fn rotate(dir: &Path, keep: usize) -> Result<()> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "gz").unwrap_or(false))
        .collect();
    files.sort_by(|a, b| b.file_name().cmp(&a.file_name())); // newest first (name = timestamp)
    for old in files.into_iter().skip(keep) {
        std::fs::remove_file(&old).with_context(|| format!("rotate {}", old.display()))?;
    }
    Ok(())
}

/// List archives newest-first: `(basename, bytes)`.
pub fn list(dir: &Path) -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = std::fs::read_dir(dir)
        .map(|it| {
            it.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|x| x == "gz").unwrap_or(false))
                .filter_map(|p| {
                    let n = p.file_name()?.to_str()?.to_string();
                    let sz = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                    Some((n, sz))
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v
}

/// Resolve `backups/<name>` guarding against traversal — only basenames
/// ending `.tar.gz` are legal.
pub fn resolve(dir: &Path, name: &str) -> Result<PathBuf> {
    if name.contains('/') || name.contains('\\') || name.contains("..") || !name.ends_with(".tar.gz") {
        bail!("bad archive name");
    }
    Ok(dir.join(name))
}
