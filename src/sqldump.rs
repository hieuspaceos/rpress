//! Streaming mysqldump parser: yields rows from
//! `INSERT INTO `t` VALUES (...),(...);` as Vec<Val>.
//! Handles mysqldump escapes (\' \n \r \t \0 \\ \"), NULL, bare numbers.

use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub enum Val {
    Null,
    Str(Vec<u8>),
    Num(String),
}

impl Val {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Val::Str(b) => std::str::from_utf8(b).ok(),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Val::Num(s) => s.trim().parse().ok(),
            Val::Str(b) => std::str::from_utf8(b).ok()?.trim().parse().ok(),
            _ => None,
        }
    }
    pub fn to_string_lossy(&self) -> Option<String> {
        match self {
            Val::Str(b) => Some(String::from_utf8_lossy(b).into_owned()),
            Val::Num(s) => Some(s.clone()),
            Val::Null => None,
        }
    }
}

/// Stream every row of INSERTs for `wanted` tables, calling f(table, vals).
pub fn rows<F>(path: &Path, wanted: &[&str], mut f: F) -> Result<u64>
where
    F: FnMut(&str, Vec<Val>) -> Result<()>,
{
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut rdr = BufReader::with_capacity(4 * 1024 * 1024, file);
    let mut total = 0u64;
    let mut line: Vec<u8> = Vec::with_capacity(8 * 1024 * 1024);

    loop {
        line.clear();
        if rdr.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        if !line.starts_with(b"INSERT INTO `") {
            continue;
        }
        let tick_start = 13; // position after `INSERT INTO \``
        let tick_end = line[tick_start..]
            .iter()
            .position(|&c| c == b'`')
            .map(|p| tick_start + p)
            .context("bad INSERT header")?;
        let table = String::from_utf8_lossy(&line[tick_start..tick_end]).into_owned();
        if !wanted.contains(&table.as_str()) {
            continue;
        }
        // Statement may span lines (very long INSERTs) — append until it
        // ends with ';' at top level. mysqldump keeps one INSERT per line
        // though, so usually this loop is a no-op.
        let mut stmt = line.clone();
        while !is_complete(&stmt) {
            line.clear();
            if rdr.read_until(b'\n', &mut line)? == 0 {
                bail!("unterminated INSERT for {}", table);
            }
            stmt.extend_from_slice(&line);
        }
        let vals_off = find_values(&stmt)?;
        parse_rows(&stmt[vals_off..], &table, &mut f, &mut total)?;
    }
    Ok(total)
}

fn is_complete(stmt: &[u8]) -> bool {
    let t = trim_end(stmt);
    t.ends_with(b";")
}

fn trim_end(b: &[u8]) -> &[u8] {
    let mut e = b.len();
    while e > 0 && b[e - 1].is_ascii_whitespace() {
        e -= 1;
    }
    &b[..e]
}

fn find_values(stmt: &[u8]) -> Result<usize> {
    stmt.windows(6)
        .position(|w| w == b"VALUES")
        .map(|p| p + 6)
        .context("INSERT without VALUES")
}

/// Parse `(...),(...);` — single pass.
fn parse_rows<F>(s: &[u8], table: &str, f: &mut F, total: &mut u64) -> Result<()>
where
    F: FnMut(&str, Vec<Val>) -> Result<()>,
{
    let mut i = 0usize;
    while i < s.len() {
        match s[i] {
            b'(' => {
                i += 1;
                let mut vals = Vec::with_capacity(24);
                loop {
                    // parse one value
                    while i < s.len() && s[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    match s.get(i).copied().context("EOF inside row")? {
                        b'\'' => {
                            i += 1;
                            let mut out: Vec<u8> = Vec::new();
                            let mut chunk = i;
                            loop {
                                match s.get(i).copied().context("unterminated string")? {
                                    b'\\' => {
                                        out.extend_from_slice(&s[chunk..i]);
                                        i += 1;
                                        let c = s.get(i).copied().context("bad escape")?;
                                        out.push(match c {
                                            b'n' => b'\n',
                                            b'r' => b'\r',
                                            b't' => b'\t',
                                            b'0' => 0,
                                            b'Z' => 0x1A,
                                            _ => c,
                                        });
                                        i += 1;
                                        chunk = i;
                                    }
                                    b'\'' => {
                                        out.extend_from_slice(&s[chunk..i]);
                                        i += 1;
                                        if s.get(i) == Some(&b'\'') {
                                            out.push(b'\'');
                                            i += 1;
                                            chunk = i;
                                        } else {
                                            vals.push(Val::Str(out));
                                            break;
                                        }
                                    }
                                    _ => i += 1,
                                }
                            }
                        }
                        b'N' => {
                            if s.get(i..i + 4) != Some(b"NULL") {
                                bail!("expected NULL");
                            }
                            i += 4;
                            vals.push(Val::Null);
                        }
                        _ => {
                            let start = i;
                            while i < s.len() && s[i] != b',' && s[i] != b')' {
                                i += 1;
                            }
                            let t = String::from_utf8_lossy(&s[start..i]).trim().to_string();
                            vals.push(if t.is_empty() { Val::Null } else { Val::Num(t) });
                        }
                    }
                    // consume , or )
                    while i < s.len() && s[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    match s.get(i).copied().context("row not closed")? {
                        b',' => i += 1,
                        b')' => {
                            i += 1;
                            break;
                        }
                        c => bail!("expected ',' or ')' got '{}'", c as char),
                    }
                }
                f(table, vals)?;
                *total += 1;
            }
            b',' => i += 1,
            b';' => break,
            c if c.is_ascii_whitespace() => i += 1,
            c => bail!("unexpected '{}'", c as char),
        }
    }
    Ok(())
}
