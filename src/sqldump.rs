//! Streaming mysqldump parser: yields rows from
//! `INSERT INTO `t` VALUES (...),(...);` as Vec<Val>.
//! Handles mysqldump escapes (\' \n \r \t \0 \\ \b \f \Z \"), NULL, bare
//! numbers, `_charset'…'`/`N'…'` string introducers, unquoted table names,
//! explicit column lists (remapped to CREATE TABLE order), multi-line
//! statements and gzip-compressed dumps (magic-byte detection).

use anyhow::{bail, Context, Result};
use std::collections::HashMap;
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
/// Rows come back in canonical column order: positional INSERTs are used
/// as-is, `INSERT INTO t (cols…)` rows are remapped to the table's
/// CREATE TABLE column order (missing columns → NULL, unknown → bail).
/// .gz input (magic bytes) is decompressed transparently.
pub fn rows<F>(path: &Path, wanted: &[&str], mut f: F) -> Result<u64>
where
    F: FnMut(&str, Vec<Val>) -> Result<()>,
{
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut peek = BufReader::with_capacity(4 * 1024 * 1024, file);
    let gz = peek
        .fill_buf()
        .map(|b| b.starts_with(&[0x1f, 0x8b]))
        .unwrap_or(false);
    let mut rdr: Box<dyn BufRead> = if gz {
        Box::new(BufReader::new(
            flate2::read::MultiGzDecoder::new(peek),
        ))
    } else {
        Box::new(peek)
    };

    let mut total = 0u64;
    let mut line: Vec<u8> = Vec::with_capacity(8 * 1024 * 1024);
    let mut schemas: HashMap<String, Vec<String>> = HashMap::new();

    loop {
        line.clear();
        if rdr.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        if let Some(table) = insert_table(&line) {
            if !wanted.contains(&table.as_str()) {
                continue;
            }
            // Statement may span lines (very long INSERTs, embedded
            // newlines inside strings) — buffer until the statement's
            // terminating ';' outside any string literal.
            let mut stmt = line.clone();
            while !is_complete(&stmt) {
                line.clear();
                if rdr.read_until(b'\n', &mut line)? == 0 {
                    bail!("unterminated INSERT for {}", table);
                }
                stmt.extend_from_slice(&line);
            }
            let (cols, vals_off) = insert_header(&stmt)
                .with_context(|| format!("bad INSERT into {table}"))?;
            let remap = remap_for(&table, cols.as_deref(), &schemas)?;
            parse_rows(&stmt[vals_off..], &table, remap.as_ref(), &mut f, &mut total)?;
        } else if starts_word(&line, b"CREATE") {
            // Buffer the DDL (needed to decode column-list INSERTs below).
            let mut stmt = line.clone();
            while !is_complete(&stmt) {
                line.clear();
                if rdr.read_until(b'\n', &mut line)? == 0 {
                    stmt.clear();
                    break;
                }
                stmt.extend_from_slice(&line);
            }
            if let Some((t, cols)) = create_table_cols(&stmt) {
                schemas.insert(t, cols);
            }
        } else if starts_word(&line, b"REPLACE")
            || (starts_word(&line, b"INSERT") && !insert_into_like(&line))
            || insert_into_like(&line)
        {
            // REPLACE INTO, INSERT DELAYED/LOW_PRIORITY/SELECT and
            // malformed INSERT INTO headers: never silently skip row
            // writes — bail so the dump variant is handled explicitly.
            let head: String = line
                .iter()
                .take_while(|&&c| c != b'\n' && c != b'\r')
                .take(80)
                .map(|&c| c as char)
                .collect();
            bail!(
                "unsupported statement (only INSERT INTO `t` VALUES is parsed): {}",
                head.trim()
            );
        }
    }
    Ok(total)
}

/// `s[i..]` starts with `w` followed by a non-identifier char, case-insensitive.
fn word(s: &[u8], i: usize, w: &[u8]) -> bool {
    s.get(i..i + w.len()).map(|x| x.eq_ignore_ascii_case(w)).unwrap_or(false)
        && !s.get(i + w.len()).map(|c| c.is_ascii_alphanumeric() || *c == b'_').unwrap_or(false)
}

/// Whitespace plus SQL comments (`-- …`, `/* … */`) between tokens at the
/// start of a line — phpMyAdmin dumps precede statements with comments.
fn skip_ws_comments(s: &[u8], mut i: usize) -> usize {
    loop {
        i = skip_ws(s, i);
        if s.get(i..i + 2) == Some(b"--") {
            while i < s.len() && s[i] != b'\n' {
                i += 1;
            }
        } else if s.get(i..i + 2) == Some(b"/*") {
            match s.get(i + 2..).and_then(|t| t.windows(2).position(|w| w == b"*/")) {
                Some(p) => i += 2 + p + 2,
                None => return s.len(), // unterminated comment eats the line
            }
        } else {
            return i;
        }
    }
}

fn skip_ws(s: &[u8], mut i: usize) -> usize {
    while s.get(i).map(|c| c.is_ascii_whitespace()).unwrap_or(false) {
        i += 1;
    }
    i
}

fn starts_word(s: &[u8], w: &[u8]) -> bool {
    word(s, skip_ws_comments(s, 0), w)
}

/// Backticked or bare identifier (possibly `db.table` qualified — last part
/// wins). Returns (name, next offset).
fn ident(s: &[u8], mut i: usize) -> Result<(String, usize)> {
    let mut last;
    loop {
        i = skip_ws(s, i);
        match s.get(i).copied().context("expected identifier")? {
            b'`' => {
                i += 1;
                let st = i;
                while i < s.len() && s[i] != b'`' {
                    i += 1;
                }
                if i == s.len() {
                    bail!("unterminated `ident`");
                }
                last = String::from_utf8_lossy(&s[st..i]).into_owned();
                i += 1;
            }
            c if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' => {
                let st = i;
                while i < s.len()
                    && (s[i].is_ascii_alphanumeric() || s[i] == b'_' || s[i] == b'$')
                {
                    i += 1;
                }
                last = String::from_utf8_lossy(&s[st..i]).into_owned();
            }
            c => bail!("expected identifier, got '{}'", c as char),
        }
        if s.get(i) == Some(&b'.') {
            i += 1;
            continue;
        }
        return Ok((last, i));
    }
}

/// Lenient INSERT-header scan for the per-line pre-filter. Returns the
/// (possibly qualified) table name, or None when the line isn't an
/// `INSERT [IGNORE] INTO <ident>` opener.
fn insert_table(line: &[u8]) -> Option<String> {
    let mut i = skip_ws_comments(line, 0);
    if !word(line, i, b"INSERT") {
        return None;
    }
    i += 6;
    i = skip_ws(line, i);
    if word(line, i, b"IGNORE") {
        i += 6;
        i = skip_ws(line, i);
    }
    if !word(line, i, b"INTO") {
        return None;
    }
    i += 4;
    ident(line, i).ok().map(|(name, _)| name)
}

/// True when the line is an `INSERT [IGNORE] INTO` opener — used to flag
/// INSERT variants (DELAYED, … SELECT, malformed headers) as unsupported
/// rather than silently skipping them.
fn insert_into_like(line: &[u8]) -> bool {
    let mut i = skip_ws_comments(line, 0);
    if !word(line, i, b"INSERT") {
        return false;
    }
    i += 6;
    i = skip_ws(line, i);
    if word(line, i, b"IGNORE") {
        i += 6;
        i = skip_ws(line, i);
    }
    word(line, i, b"INTO")
}

/// Strict INSERT header on the buffered statement:
/// `INSERT [IGNORE] INTO ident [(col,…)] VALUES` → (col list, values offset).
fn insert_header(stmt: &[u8]) -> Result<(Option<Vec<String>>, usize)> {
    let mut i = skip_ws(stmt, 0);
    if !word(stmt, i, b"INSERT") {
        bail!("not an INSERT");
    }
    i += 6;
    i = skip_ws(stmt, i);
    if word(stmt, i, b"IGNORE") {
        i += 6;
        i = skip_ws(stmt, i);
    }
    if !word(stmt, i, b"INTO") {
        bail!("INSERT without INTO");
    }
    i += 4;
    let (_t, j) = ident(stmt, i)?;
    i = skip_ws(stmt, j);
    let mut cols = None;
    if stmt.get(i) == Some(&b'(') {
        i += 1;
        let mut list = Vec::new();
        loop {
            let (c, j) = ident(stmt, i)?;
            list.push(c);
            i = skip_ws(stmt, j);
            match stmt.get(i).copied().context("column list not closed")? {
                b',' => i += 1,
                b')' => {
                    i += 1;
                    break;
                }
                c => bail!("expected ',' or ')' in column list, got '{}'", c as char),
            }
        }
        i = skip_ws(stmt, i);
        cols = Some(list);
    }
    if !word(stmt, i, b"VALUES") {
        bail!("INSERT without VALUES");
    }
    i += 6;
    Ok((cols, i))
}

/// Column-name → canonical-order mapping for an INSERT with an explicit
/// column list. Pairs are (canonical idx, source idx).
struct Remap {
    out_len: usize,
    pairs: Vec<(usize, usize)>,
}

fn remap_for(
    table: &str,
    cols: Option<&[String]>,
    schemas: &HashMap<String, Vec<String>>,
) -> Result<Option<Remap>> {
    let Some(cols) = cols else { return Ok(None) };
    let schema = schemas
        .get(table)
        .with_context(|| format!("{table}: INSERT names columns but dump has no CREATE TABLE to map them against"))?;
    let mut pairs = Vec::with_capacity(cols.len());
    for (src, c) in cols.iter().enumerate() {
        match schema.iter().position(|s| s == c) {
            Some(dst) => pairs.push((dst, src)),
            None => bail!("{table}: INSERT lists unknown column {c:?}"),
        }
    }
    Ok(Some(Remap { out_len: schema.len(), pairs }))
}

/// Column names of `CREATE TABLE t (…)` in declaration order, or None when
/// the statement isn't a parseable CREATE TABLE (e.g. CREATE TABLE … SELECT).
fn create_table_cols(stmt: &[u8]) -> Option<(String, Vec<String>)> {
    let mut i = skip_ws(stmt, 0);
    if !word(stmt, i, b"CREATE") {
        return None;
    }
    i += 6;
    i = skip_ws(stmt, i);
    if word(stmt, i, b"TEMPORARY") {
        i += 9;
        i = skip_ws(stmt, i);
    }
    if !word(stmt, i, b"TABLE") {
        return None;
    }
    i += 5;
    i = skip_ws(stmt, i);
    if word(stmt, i, b"IF") {
        // IF NOT EXISTS
        let j = skip_ws(stmt, i + 2);
        if word(stmt, j, b"NOT") {
            i = skip_ws(stmt, j + 3);
            if word(stmt, i, b"EXISTS") {
                i += 6;
            }
        }
    }
    let (name, j) = ident(stmt, i).ok()?;
    i = skip_ws(stmt, j);
    if stmt.get(i) != Some(&b'(') {
        return None; // CREATE TABLE … AS SELECT / LIKE
    }
    i += 1;
    let mut cols = Vec::new();
    let mut depth = 1usize;
    let mut start = i;
    loop {
        match stmt.get(i).copied() {
            None => return None,
            Some(b'\'') | Some(b'"') => {
                i = skip_string(stmt, i)?;
            }
            Some(b'`') => {
                i += 1;
                while i < stmt.len() && stmt[i] != b'`' {
                    i += 1;
                }
                i += 1;
            }
            Some(b'(') => {
                depth += 1;
                i += 1;
            }
            Some(b')') => {
                if depth == 1 {
                    add_col_def(&stmt[start..i], &mut cols);
                    return Some((name, cols));
                }
                depth -= 1;
                i += 1;
            }
            Some(b',') if depth == 1 => {
                add_col_def(&stmt[start..i], &mut cols);
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
}

/// First token of a column definition → column name; constraint lines
/// (PRIMARY KEY, KEY, INDEX, UNIQUE, CONSTRAINT, FOREIGN, CHECK, FULLTEXT,
/// SPATIAL) contribute nothing.
fn add_col_def(def: &[u8], cols: &mut Vec<String>) {
    const CONSTRAINT: &[&[u8]] = &[
        b"PRIMARY", b"KEY", b"INDEX", b"UNIQUE", b"CONSTRAINT", b"FOREIGN",
        b"CHECK", b"FULLTEXT", b"SPATIAL",
    ];
    let i = skip_ws(def, 0);
    match def.get(i).copied() {
        Some(b'`') => {
            if let Some(e) = def[i + 1..].iter().position(|&c| c == b'`') {
                cols.push(String::from_utf8_lossy(&def[i + 1..i + 1 + e]).into_owned());
            }
        }
        Some(c) if c.is_ascii_alphabetic() || c == b'_' => {
            let mut e = i;
            while e < def.len()
                && (def[e].is_ascii_alphanumeric() || def[e] == b'_' || def[e] == b'$')
            {
                e += 1;
            }
            let w = &def[i..e];
            if !CONSTRAINT.iter().any(|k| w.eq_ignore_ascii_case(k)) {
                cols.push(String::from_utf8_lossy(w).into_owned());
            }
        }
        _ => {}
    }
}

/// Statement ends with `;` at top level — no unclosed `'…'` string.
/// `\'`, `\\` and `''` escapes are honored.
fn is_complete(stmt: &[u8]) -> bool {
    let mut in_str = false;
    let mut in_tick = false;
    let mut end = 0usize; // offset just past the top-level ';'
    let mut i = 0;
    while i < stmt.len() {
        match stmt[i] {
            b'\'' if !in_tick => {
                if in_str && stmt.get(i + 1) == Some(&b'\'') {
                    i += 2; // '' inside a string is an escaped quote
                    continue;
                }
                in_str = !in_str;
            }
            b'`' if !in_str => in_tick = !in_tick,
            b'\\' if in_str => i += 1, // skip the escaped byte
            b';' if !in_str && !in_tick => end = i + 1,
            _ => {}
        }
        i += 1;
    }
    end > 0 && !in_str && !in_tick && stmt[end..].iter().all(|c| c.is_ascii_whitespace())
}

/// Skip a `'…'` or `"…"` literal (escapes: backslash + doubled quote).
fn skip_string(s: &[u8], mut i: usize) -> Option<usize> {
    let q = s[i];
    i += 1;
    while i < s.len() {
        match s[i] {
            b'\\' => i += 2,
            c if c == q => {
                if s.get(i + 1) == Some(&q) {
                    i += 2;
                } else {
                    return Some(i + 1);
                }
            }
            _ => i += 1,
        }
    }
    None
}

fn parse_quoted(s: &[u8], i: &mut usize, latin1: bool) -> Result<Val> {
    *i += 1; // opening '
    let mut out: Vec<u8> = Vec::new();
    let mut chunk = *i;
    loop {
        match s.get(*i).copied().context("unterminated string")? {
            b'\\' => {
                out.extend_from_slice(&s[chunk..*i]);
                *i += 1;
                let c = s.get(*i).copied().context("bad escape")?;
                out.push(match c {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'0' => 0,
                    b'b' => 0x08,
                    b'f' => 0x0C,
                    b'Z' => 0x1A,
                    _ => c,
                });
                *i += 1;
                chunk = *i;
            }
            b'\'' => {
                out.extend_from_slice(&s[chunk..*i]);
                *i += 1;
                if s.get(*i) == Some(&b'\'') {
                    out.push(b'\'');
                    *i += 1;
                    chunk = *i;
                } else {
                    return Ok(Val::Str(if latin1 { latin1_to_utf8(&out) } else { out }));
                }
            }
            _ => *i += 1,
        }
    }
}

fn latin1_to_utf8(b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(b.len() * 2);
    for &c in b {
        if c < 0x80 {
            out.push(c);
        } else {
            out.push(0xC0 | (c >> 6));
            out.push(0x80 | (c & 0x3F));
        }
    }
    out
}

/// One value at s[*i]: `'…'`, `_charset'…'`, `N'…'`, NULL or a bare token.
fn parse_value(s: &[u8], i: &mut usize) -> Result<Val> {
    while *i < s.len() && s[*i].is_ascii_whitespace() {
        *i += 1;
    }
    match s.get(*i).copied().context("EOF inside row")? {
        b'\'' => parse_quoted(s, i, false),
        b'_' => {
            // `_charset'string'` introducer.
            let mut e = *i + 1;
            while e < s.len() && (s[e].is_ascii_alphanumeric() || s[e] == b'_') {
                e += 1;
            }
            if s.get(e) == Some(&b'\'') {
                let cs = s.get(*i + 1..e).unwrap_or(b"");
                let latin1 = cs.eq_ignore_ascii_case(b"latin1");
                *i = e;
                parse_quoted(s, i, latin1)
            } else {
                bail!(
                    "expected string after '_{}' introducer",
                    String::from_utf8_lossy(s.get(*i + 1..e).unwrap_or(b""))
                );
            }
        }
        b'N' | b'n' => {
            if s.get(*i + 1) == Some(&b'\'') {
                *i += 1;
                parse_quoted(s, i, false)
            } else {
                let ok = s
                    .get(*i + 1..*i + 4)
                    .map(|t| t.eq_ignore_ascii_case(b"ULL"))
                    .unwrap_or(false)
                    && !s
                        .get(*i + 4)
                        .map(|c| c.is_ascii_alphanumeric() || *c == b'_')
                        .unwrap_or(false);
                if !ok {
                    bail!("expected NULL or N'string'");
                }
                *i += 4;
                Ok(Val::Null)
            }
        }
        _ => {
            let start = *i;
            while *i < s.len() && s[*i] != b',' && s[*i] != b')' {
                *i += 1;
            }
            let t = String::from_utf8_lossy(&s[start..*i]).trim().to_string();
            Ok(if t.is_empty() { Val::Null } else { Val::Num(t) })
        }
    }
}

/// Parse `(...),(...);` — single pass.
fn parse_rows<F>(s: &[u8], table: &str, remap: Option<&Remap>, f: &mut F, total: &mut u64) -> Result<()>
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
                    vals.push(parse_value(s, &mut i)?);
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
                match remap {
                    Some(r) => {
                        if vals.len() != r.pairs.len() {
                            bail!(
                                "{table}: row has {} values for {} listed columns",
                                vals.len(),
                                r.pairs.len()
                            );
                        }
                        let mut out: Vec<Val> = Vec::new();
                        out.resize_with(r.out_len, || Val::Null);
                        for (dst, src) in r.pairs.iter() {
                            out[*dst] = std::mem::replace(&mut vals[*src], Val::Null);
                        }
                        f(table, out)?;
                    }
                    None => f(table, vals)?,
                }
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

/// Detect the dump's table prefix: first `INSERT INTO `<p>posts`` (or the
/// CREATE TABLE for it). Falls back to `wp_` when nothing turns up —
/// a posts-less dump imports empty either way.
pub fn detect_prefix(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut peek = BufReader::with_capacity(1024 * 1024, file);
    let gz = peek
        .fill_buf()
        .map(|b| b.starts_with(&[0x1f, 0x8b]))
        .unwrap_or(false);
    let mut rdr: Box<dyn BufRead> = if gz {
        Box::new(BufReader::new(flate2::read::MultiGzDecoder::new(peek)))
    } else {
        Box::new(peek)
    };
    /* Collect table names first — plugin tables can end in `posts`
       (`wp_42426_pmxe_posts`) and sort BEFORE the core table, so a
       first-match prefix is wrong. The real posts prefix also owns
       `<p>postmeta`. */
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut line = Vec::with_capacity(1024 * 1024);
    while rdr.read_until(b'\n', &mut line)? > 0 {
        let s = String::from_utf8_lossy(&line);
        for kw in ["INSERT INTO `", "CREATE TABLE `", "INSERT INTO ", "CREATE TABLE "] {
            if let Some(i) = s.find(kw) {
                let name: String = s[i + kw.len()..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
                    .collect();
                if !name.is_empty() {
                    names.insert(name);
                }
                break;
            }
        }
        line.clear();
    }
    let mut candidates: Vec<String> = names
        .iter()
        .filter_map(|n| n.strip_suffix("posts").map(|p| p.to_string()))
        .filter(|p| !p.is_empty() && names.contains(&format!("{p}postmeta")))
        .collect();
    /* Prefer the shortest prefix — `wp_` over `wp_42426_` over plugin-y
       longer ones when several satisfy the pair check. */
    candidates.sort_by_key(|p| p.len());
    Ok(candidates.into_iter().next().unwrap_or_else(|| "wp_".to_string()))
}
