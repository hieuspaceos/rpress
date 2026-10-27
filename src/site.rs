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

/// POSIX single-quote a string for embedding inside remote/local shell
/// commands — `'a'\''b'` style. Every attacker-influenced value that must
/// reach a shell string (wp-config creds, remote paths, dest dirs) goes
/// through this; anything reachable argv-first skips the shell entirely.
pub(crate) fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// `^[a-z0-9][a-z0-9-]*$` — the slug doubles as a filename under
/// `sites/` + `work/`, so path-traversal shapes (`../`, `.`, `/`) must
/// never pass.
pub(crate) fn valid_slug(site: &str) -> bool {
    let mut it = site.chars();
    matches!(it.next(), Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit())
        && it.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}


#[derive(Default, Debug)]
pub struct SiteEnv {
    pub site: String,
    pub name: String,
    pub host: String,
    pub bind: String,
    pub local_url: String,
    // deprecated: sourced via KindBases at entry points; remove phase 05
    pub tour_base: String,
    /// Origin URL the dump came from (parity target + asset fallback).
    pub site_url: String,
    /// Comma-separated PhpBridge opt-in slugs (`PLUGINS_PHP=`). Empty =
    /// bridge fully off — no runtime surface unless explicitly listed.
    pub plugins_php: String,
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
        if !valid_slug(site) {
            bail!("invalid site slug {site:?} — want [a-z0-9][a-z0-9-]*");
        }
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
        // SITE= inside the file must match the filename stem — otherwise
        // work/<SITE>/ would resolve somewhere the slug never pointed.
        let site_in_file = get("SITE");
        if site_in_file != site {
            bail!(
                "{} declares SITE={site_in_file} but lives at sites/{site}.env — rename the file or fix SITE=",
                path.display()
            );
        }
        Ok(Self {
            site: site_in_file,
            name: get("NAME"),
            host: get("HOST"),
            bind: get("BIND"),
            local_url: get("LOCAL_URL"),
            tour_base: get("TOUR_BASE"),
            site_url: get("SITE_URL"),
            plugins_php: get("PLUGINS_PHP"),
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
        if !valid_slug(&self.site) {
            bail!("invalid site slug {:?} — want [a-z0-9][a-z0-9-]*", self.site);
        }
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
        s.push_str(&format!("PLUGINS_PHP={}\n", self.plugins_php));
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

    /// Short human label for diagnostics — "ct4" or "user@host".
    pub fn label(&self) -> String {
        match self {
            Remote::Vps { name, .. } => name.clone(),
            Remote::Ssh { target } => target.clone(),
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

/// Numbered menu; returns the picked index. Fails on empty list or bad
/// input. Non-TTY with more than one option is never auto-picked — bail
/// naming `flag_hint`, the flag that would have skipped this prompt.
pub fn choose(label: &str, items: &[String], flag_hint: &str) -> Result<usize> {
    if items.is_empty() {
        bail!("{label}: nothing to choose from");
    }
    if items.len() == 1 {
        eprintln!("{label}: {} (only option)", items[0]);
        return Ok(0);
    }
    if !tty() {
        bail!("{label}: {} options, non-interactive — pass --{flag_hint} to pick", items.len());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_round_trip() {
        // plain values pass through as 'value'
        assert_eq!(shell_quote("/var/www/site"), "'/var/www/site'");
        // embedded quotes use the POSIX '"'"' idiom — round-trips under sh
        assert_eq!(shell_quote("x';id;'"), "'x'\\'';id;'\\'''");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        // metachars are inert inside single quotes
        assert_eq!(shell_quote("a; b$(c)`d`e&f|g>h<i\n"), "'a; b$(c)`d`e&f|g>h<i\n'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn shell_quote_survives_sh() {
        // real round-trip: the quoted string must arrive at argv verbatim
        for raw in ["x';id;'", "a b", "$(touch /tmp/x)", "a\\b'c\"d", "*~?[]{}", "\t"] {
            let o = run(Command::new("sh").args(["-c", &format!("printf %s {}", shell_quote(raw))])).unwrap();
            assert!(o.ok);
            assert_eq!(o.stdout, raw, "raw={raw:?}");
        }
    }

    #[test]
    fn slug_accept_reject() {
        for ok in ["dmc", "blueskies", "aucoeur-test", "x", "a1-b2", "0start"] {
            assert!(valid_slug(ok), "{ok}");
        }
        for bad in ["", "-lead", "A", "a_b", "a.b", "..", "../etc", "a/b", "a b", "a;b", "a'b"] {
            assert!(!valid_slug(bad), "{bad}");
        }
    }

    #[test]
    fn load_rejects_traversal_and_mismatch() {
        let root = std::env::temp_dir().join(format!("rpress-site-test-{}", std::process::id()));
        std::fs::create_dir_all(root.join("sites")).unwrap();
        // slug rejected before touching the filesystem
        assert!(SiteEnv::load(&root, "../etc").is_err());
        assert!(SiteEnv::load(&root, "Bad Slug").is_err());
        // filename stem vs SITE= mismatch is a hard error
        std::fs::write(root.join("sites/ok.env"), "SITE=other\nBIND=127.0.0.1:1\n").unwrap();
        assert!(SiteEnv::load(&root, "ok").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
