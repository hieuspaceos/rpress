//! `rpress site …` — site-level workflow shared bits.
//!
//! A site is described by `sites/<name>.env` — the same KEY=VALUE format
//! `scripts/e2e-site.sh` reads, so a site created by `site init` works with
//! both the CLI subcommands and the shell fallback. State lives in
//! `work/<site>/` and is fully disposable.

use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ---------- env file ----------

pub fn env_path(root: &Path, site: &str) -> PathBuf {
    root.join("sites").join(format!("{site}.env"))
}

#[derive(Default, Debug)]
pub struct SiteEnv {
    pub site: String,
    pub name: String,
    pub host: String,
    pub bind: String,
    pub local_url: String,
    pub tour_base: String,
    /// Origin URL the dump came from (parity target + asset fallback).
    pub site_url: String,
    pub clone_source_ok: bool,
    // pull source — exactly one family populated
    /// vps.sh hosts.conf alias (password/keyfile helper at ../vps-ssh)
    pub vps_host: Option<String>,
    /// plain openssh target user@host (keys/agent)
    pub ssh_target: Option<String>,
    pub wp_root: Option<String>,
    pub dump_remote: Option<String>,
    pub dump_local: Option<PathBuf>,
    pub wp_content_local: Option<PathBuf>,
}

impl SiteEnv {
    pub fn load(root: &Path, site: &str) -> Result<Self> {
        let path = env_path(root, site);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("site env not found: {}", path.display()))?;
        let mut kv = BTreeMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                kv.insert(
                    k.trim().to_string(),
                    v.trim().trim_matches('"').trim_matches('\'').to_string(),
                );
            }
        }
        let get = |k: &str| kv.get(k).cloned().unwrap_or_default();
        let opt = |k: &str| kv.get(k).filter(|v| !v.is_empty()).cloned();
        Ok(Self {
            site: get("SITE"),
            name: get("NAME"),
            host: get("HOST"),
            bind: get("BIND"),
            local_url: get("LOCAL_URL"),
            tour_base: get("TOUR_BASE"),
            site_url: get("SITE_URL"),
            clone_source_ok: get("CLONE_SOURCE_OK") == "yes",
            vps_host: opt("VPS_HOST"),
            ssh_target: opt("SSH_TARGET"),
            wp_root: opt("WP_ROOT"),
            dump_remote: opt("DUMP_REMOTE"),
            dump_local: opt("DUMP_LOCAL").map(PathBuf::from),
            wp_content_local: opt("WP_CONTENT_LOCAL").map(PathBuf::from),
        })
    }

    pub fn save(&self, root: &Path) -> Result<PathBuf> {
        let dir = root.join("sites");
        std::fs::create_dir_all(&dir)?;
        let path = env_path(root, &self.site);
        let mut s = String::new();
        let q = |v: &str| format!("\"{}\"", v.replace('"', "\\\""));
        s.push_str(&format!("SITE={}\n", self.site));
        s.push_str(&format!("NAME={}\n", q(&self.name)));
        s.push_str(&format!("HOST={}\n", self.host));
        s.push_str(&format!("BIND={}\n", self.bind));
        s.push_str(&format!("LOCAL_URL={}\n", self.local_url));
        s.push_str(&format!("TOUR_BASE={}\n", self.tour_base));
        s.push_str(&format!("SITE_URL={}\n", self.site_url));
        s.push_str(&format!(
            "CLONE_SOURCE_OK={}\n",
            if self.clone_source_ok { "yes" } else { "no" }
        ));
        if let Some(t) = &self.vps_host {
            s.push_str(&format!("VPS_HOST={t}\n"));
        }
        if let Some(t) = &self.ssh_target {
            s.push_str(&format!("SSH_TARGET={t}\n"));
        }
        if let Some(r) = &self.wp_root {
            s.push_str(&format!("WP_ROOT={r}\n"));
        }
        if let Some(d) = &self.dump_remote {
            s.push_str(&format!("DUMP_REMOTE={d}\n"));
        }
        if let Some(d) = &self.dump_local {
            s.push_str(&format!("DUMP_LOCAL={}\n", d.display()));
        }
        if let Some(d) = &self.wp_content_local {
            s.push_str(&format!("WP_CONTENT_LOCAL={}\n", d.display()));
        }
        std::fs::write(&path, s)?;
        Ok(path)
    }

    pub fn port(&self) -> u16 {
        self.bind
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8400)
    }

    pub fn has_remote(&self) -> bool {
        self.vps_host.is_some() || self.ssh_target.is_some()
    }
}

// ---------- remote shells ----------

/// How to reach the source host. `vps.sh` aliases use the sibling helper
/// (password/keyfile auth via .askpass); everything else is plain openssh.
pub enum Remote {
    Vps { name: String, script: PathBuf },
    Ssh { target: String },
}

impl Remote {
    pub fn resolve(target: &str, root: &Path) -> Remote {
        let script = std::env::var("VPS_SSH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root.join("../vps-ssh/vps.sh"));
        if script.is_file() && !target.contains('@') {
            if let Ok(o) = run(Command::new(&script).arg("list")) {
                let known = o
                    .stdout
                    .lines()
                    .any(|l| l.split_whitespace().next() == Some(target));
                if known {
                    return Remote::Vps { name: target.into(), script };
                }
            }
        }
        Remote::Ssh { target: target.into() }
    }

    pub fn exec(&self, remote: &str) -> Result<Out> {
        match self {
            Remote::Vps { name, script } => {
                run(Command::new(script).args([name.as_str(), remote]))
            }
            Remote::Ssh { target } => ssh(target, remote),
        }
    }

    pub fn lines(&self, remote: &str) -> Result<Vec<String>> {
        let o = self.exec(remote)?;
        if !o.ok {
            bail!("{}", o.stderr.trim());
        }
        Ok(o.stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }

    /// (env key, value) this remote persists as — VPS_HOST keeps e2e-site.sh working.
    pub fn env_pair(&self) -> (&'static str, String) {
        match self {
            Remote::Vps { name, .. } => ("VPS_HOST", name.clone()),
            Remote::Ssh { target } => ("SSH_TARGET", target.clone()),
        }
    }
}

// ---------- interactive prompts ----------

pub fn tty() -> bool {
    std::io::stdin().is_terminal()
}

fn read_line() -> Result<String> {
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

pub fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(d) => eprint!("{label} [{d}]: "),
        None => eprint!("{label}: "),
    }
    let v = read_line()?;
    Ok(if v.is_empty() {
        default.unwrap_or_default().to_string()
    } else {
        v
    })
}

pub fn confirm(q: &str, default: bool) -> Result<bool> {
    let hint = if default { "Y/n" } else { "y/N" };
    eprint!("{q} [{hint}]: ");
    let v = read_line()?.to_lowercase();
    Ok(match v.as_str() {
        "" => default,
        "y" | "yes" => true,
        _ => false,
    })
}

/// Numbered menu; returns the picked index. Fails on empty list or bad input.
pub fn choose(label: &str, items: &[String]) -> Result<usize> {
    if items.is_empty() {
        bail!("{label}: nothing to choose from");
    }
    if items.len() == 1 {
        eprintln!("{label}: {} (only option)", items[0]);
        return Ok(0);
    }
    for (i, it) in items.iter().enumerate() {
        eprintln!("  {}) {}", i + 1, it);
    }
    loop {
        let v = prompt(label, Some("1"))?;
        if let Ok(n) = v.parse::<usize>() {
            if (1..=items.len()).contains(&n) {
                return Ok(n - 1);
            }
        }
        eprintln!("enter a number 1-{}", items.len());
    }
}

/// Interactive value-or-flag: use `flag` when set, prompt when on a TTY,
/// otherwise bail naming the flag — the AI/non-interactive contract.
pub fn need(flag: Option<String>, flag_name: &str, label: &str, default: Option<&str>) -> Result<String> {
    if let Some(v) = flag {
        return Ok(v);
    }
    if tty() {
        return prompt(label, default);
    }
    bail!("missing --{flag_name} (non-interactive: pass every required flag)")
}

// ---------- process helpers ----------

pub struct Out {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn run(cmd: &mut Command) -> Result<Out> {
    let o = cmd
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("spawn {:?}", cmd.get_program()))?;
    Ok(Out {
        ok: o.status.success(),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    })
}

/// BatchMode ssh — keys/agent only, never prompts for a password.
pub fn ssh(target: &str, remote: &str) -> Result<Out> {
    run(Command::new("ssh").args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=10",
        target,
        remote,
    ]))
}
