//! `rpress site serve` — the sandbox run.
//!
//! Preflights the site (db, cache, port), prints a test checklist with the
//! real per-language URLs and where admin credentials live, then serves.
//!
//! `--smoke` is the machine variant: spawn the server on a throwaway port,
//! probe the key routes, kill it, and report — the "does it work at all"
//! gate a user or an agent can run without opening a browser.

use crate::admin;
use crate::db::conn;
use crate::site::{self, SiteEnv};
#[allow(unused_imports)]
use crate::sql;
use anyhow::{bail, Context, Result};
use rusqlite::params;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct ServeArgs {
    pub site: String,
    pub bind: Option<String>,
    pub smoke: bool,
}

pub fn run(a: ServeArgs) -> Result<()> {
    let root = std::env::current_dir()?;
    let env = SiteEnv::load(&root, &a.site)?;
    let work = root.join("work").join(&env.site);
    let db = work.join("rpress.db");
    let cache = work.join("cache");
    let www = work.join("www");
    let templates = root.join("templates");
    let bind = a.bind.unwrap_or_else(|| env.bind.clone());

    // preflight — fail fast with the fix, not mid-request
    if !db.is_file() {
        bail!("no {} — run `rpress site build {}` first", db.display(), env.site);
    }
    if !cache.join("index.html").is_file() {
        eprintln!("WARN no generated cache — run `rpress site build {} --only generate`", env.site);
    }
    if !a.smoke && port_busy(&bind)? {
        bail!("{bind} already in use — pass --bind <addr:port> or stop the other process");
    }

    checklist(&env, &db, &bind)?;

    if a.smoke {
        return smoke(&env, &db, &templates, &cache, &www);
    }

    let addr: std::net::SocketAddr = bind.parse().context("--bind must be addr:port")?;
    let www_opt = Some(www);
    tokio::runtime::Runtime::new()?.block_on(admin::serve(
        db,
        templates,
        cache,
        www_opt.unwrap(),
        addr,
        env.site.clone(),
        env.host.clone(),
        env.name.clone(),
        env.tour_base.clone(),
    ))
}

fn port_busy(bind: &str) -> Result<bool> {
    let addr: std::net::SocketAddr = bind.parse().context("--bind must be addr:port")?;
    Ok(TcpListener::bind(addr).is_err())
}

/// Print the sandbox checklist — real URLs per configured language.
fn checklist(env: &SiteEnv, db: &PathBuf, bind: &str) -> Result<()> {
    let port = bind.rsplit(':').next().unwrap_or("8400");
    let base = format!("http://localhost:{port}");
    let (primary, langs) = langs(db);
    eprintln!("\nsite {base}");
    for lang in langs.split(',') {
        if lang.is_empty() {
            continue;
        }
        if lang == primary {
            eprintln!("  /{lang:>3} → {base}/");
        } else {
            eprintln!("  /{lang:>3} → {base}/{lang}/");
        }
    }
    eprintln!("  search → {base}/recherche/");
    eprintln!("  admin  → {base}/admin   (creds: work/{}/credentials.txt)", env.site);
    eprintln!("checklist: SPA nav · menus · search · sidebar widgets · admin login · edit post · comment moderation\n");
    Ok(())
}

fn langs(db: &PathBuf) -> (String, String) {
    let Ok(c) = conn(db) else { return ("fr".into(), "fr".into()) };
    let get = |k: &str| -> Option<String> {
        c.query_row(sql!("render__select_value_from_settings"), params![k], |r| r.get(0)).ok()
    };
    let primary = get("primary_lang").unwrap_or_else(|| "fr".into());
    let langs = get("languages").unwrap_or_else(|| primary.clone());
    (primary, langs)
}

// ---------- smoke test ----------

fn smoke(env: &SiteEnv, db: &PathBuf, templates: &PathBuf, cache: &PathBuf, www: &PathBuf) -> Result<()> {
    // throwaway port — smoke never collides with a running sandbox
    let port = {
        let l = TcpListener::bind("127.0.0.1:0")?;
        l.local_addr()?.port()
    };
    let bind = format!("127.0.0.1:{port}");
    let exe = std::env::current_exe()?;
    let mut child = Command::new(exe)
        .args([
            "serve",
            "--bind", &bind,
            "--db", &db.display().to_string(),
            "--templates", &templates.display().to_string(),
            "--out", &cache.display().to_string(),
            "--www", &www.display().to_string(),
            "--host", "127.0.0.1",
            "--site", &env.site,
            "--name", &env.name,
            "--tour-base", &env.tour_base,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn rpress serve")?;

    let ready = wait_port(&bind, Duration::from_secs(30));
    let result = if ready {
        probe_routes(env, db, port)
    } else {
        bail!("server did not listen on {bind} within 30s");
    };
    let _ = child.kill();
    let _ = child.wait();
    result
}

fn wait_port(bind: &str, timeout: Duration) -> bool {
    let addr: std::net::SocketAddr = match bind.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    false
}

fn probe_routes(_env: &SiteEnv, db: &PathBuf, port: u16) -> Result<()> {
    let (primary, langs) = langs(db);
    let mut routes = vec!["/".to_string(), "/nav.js".into(), "/front.css".into(), "/admin".into()];
    for lang in langs.split(',') {
        if lang != primary && !lang.is_empty() {
            routes.push(format!("/{lang}/"));
        }
    }
    if db_has_pagefind(db) {
        routes.push("/recherche/".into());
    }
    let base = format!("http://localhost:{port}");
    let mut failed = vec![];
    for r in &routes {
        let code = http_code(&format!("{base}{r}"));
        let ok = (200..400).contains(&code);
        eprintln!("  {} {:<6} {}", if ok { "ok " } else { "FAIL" }, code, r);
        if !ok {
            failed.push(r.clone());
        }
    }
    if !failed.is_empty() {
        bail!("{} route(s) failed: {}", failed.len(), failed.join(" "));
    }
    eprintln!("smoke: {} routes ok", routes.len());
    Ok(())
}

fn db_has_pagefind(db: &PathBuf) -> bool {
    db.parent()
        .map(|w| w.join("cache/pagefind/pagefind.js").is_file())
        .unwrap_or(false)
}

fn http_code(url: &str) -> u16 {
    site::run(Command::new("curl").args([
        "-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "15", url,
    ]))
    .map(|o| o.stdout.trim().parse().unwrap_or(0))
    .unwrap_or(0)
}
