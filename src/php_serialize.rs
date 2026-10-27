//! Minimal PHP `unserialize` → serde_json::Value.
//! Supports N;, b:, i:, d:, s:, a:, O: (objects → {"__class": n, ...props}).
//! References (r:/R:) resolve as null — WP payloads don't rely on them.

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};

/// `Vec::with_capacity` cap for declared array/object sizes — a hostile
/// `a:999999999:` must fail cleanly, not abort on a ~GB allocation. Small
/// (4096): declared counts multiply across nesting levels; Vec grows anyway.
const MAX_PREALLOC: usize = 4_096;

/// Recursion cap — `a:`/`O:` nest via `value()`; crafted deep input would
/// overflow the stack before any length check fires. 256 (not 512): the
/// `value_inner` frame is large enough that 512 live frames overflow the
/// default 2 MiB test-thread stack.
const MAX_DEPTH: usize = 256;

pub fn unserialize(input: &[u8]) -> Result<Value> {
    let mut p = Parser { b: input, i: 0, depth: 0 };
    let v = p.value()?;
    // Anything past (or overrun beyond) the top-level value is malformed
    // input, not a second document — trailing ASCII whitespace is tolerated
    // (meta blobs written with a final newline still parse).
    if p.i > p.b.len() || p.b[p.i..].iter().any(|b| !b.is_ascii_whitespace()) {
        bail!("serialized value does not consume the input exactly");
    }
    Ok(v)
}

/// Serialize `serde_json::Value` back to PHP `serialize` bytes.
/// Object → `a:` assoc; Array → `a:` seq; Number → `i:`/`d:`; Bool → `b:`;
/// String → `s:` (byte length); Null → `N;`. `__class` objects round-trip
/// as plain assoc arrays — WP option blobs never rely on class names.
pub fn serialize(v: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    write_value(&mut out, v);
    out
}

fn write_value(out: &mut Vec<u8>, v: &Value) {
    match v {
        Value::Null => out.extend_from_slice(b"N;"),
        Value::Bool(b) => {
            out.extend_from_slice(b"b:");
            out.extend_from_slice(if *b { b"1" } else { b"0" });
            out.push(b';');
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                out.extend_from_slice(format!("i:{i};").as_bytes());
            } else if let Some(u) = n.as_u64() {
                out.extend_from_slice(format!("i:{u};").as_bytes());
            } else if let Some(d) = n.as_f64() {
                // PHP uses serialize_precision=-1 style output; a plain
                // decimal is fine for WP payloads.
                out.extend_from_slice(format!("d:{d};").as_bytes());
            }
        }
        Value::String(s) => {
            out.extend_from_slice(format!("s:{}:\"", s.as_bytes().len()).as_bytes());
            out.extend_from_slice(s.as_bytes());
            out.extend_from_slice(b"\";");
        }
        Value::Array(a) => {
            out.extend_from_slice(format!("a:{}:{{", a.len()).as_bytes());
            for (i, item) in a.iter().enumerate() {
                write_value(out, &Value::from(i as i64));
                write_value(out, item);
            }
            out.push(b'}');
        }
        Value::Object(m) => {
            out.extend_from_slice(format!("a:{}:{{", m.len()).as_bytes());
            for (k, item) in m {
                // numeric-looking keys must serialize as i: to match PHP
                if let Ok(i) = k.parse::<i64>() {
                    write_value(out, &Value::from(i));
                } else {
                    write_value(out, &Value::String(k.clone()));
                }
                write_value(out, item);
            }
            out.push(b'}');
        }
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Result<u8> {
        self.b.get(self.i).copied().context("unexpected end of input")
    }
    fn eat(&mut self, c: u8) -> Result<()> {
        if self.peek()? == c {
            self.i += 1;
            Ok(())
        } else {
            bail!("expected '{}' at byte {}, found '{}'", c as char, self.i, self.b[self.i] as char)
        }
    }
    fn num_until(&mut self, end: u8) -> Result<i64> {
        let start = self.i;
        while self.i < self.b.len() && self.b[self.i] != end {
            self.i += 1;
        }
        let s = std::str::from_utf8(&self.b[start..self.i]).context("number not utf8")?;
        self.i += 1; // skip end
        s.parse().with_context(|| format!("bad number '{}'", s))
    }
    /// Declared element/byte count: negative values were silently widened
    /// to `usize::MAX` (alloc abort / slice-overflow panic) — bail instead.
    fn count_until(&mut self, end: u8) -> Result<usize> {
        let n = self.num_until(end)?;
        if n < 0 {
            bail!("negative declared length {n} at byte {}", self.i);
        }
        Ok(n as usize)
    }
    fn float_until(&mut self, end: u8) -> Result<f64> {
        let start = self.i;
        while self.i < self.b.len() && self.b[self.i] != end {
            self.i += 1;
        }
        let s = std::str::from_utf8(&self.b[start..self.i]).context("number not utf8")?;
        self.i += 1;
        match s {
            "NAN" => Ok(f64::NAN),
            "INF" => Ok(f64::INFINITY),
            "-INF" => Ok(f64::NEG_INFINITY),
            _ => s.parse().with_context(|| format!("bad float '{}'", s)),
        }
    }

    fn value(&mut self) -> Result<Value> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            bail!("nesting deeper than {MAX_DEPTH}");
        }
        let v = self.value_inner();
        self.depth -= 1;
        v
    }

    fn value_inner(&mut self) -> Result<Value> {
        match self.peek()? {
            b'N' => {
                self.i += 2;
                Ok(Value::Null)
            }
            b'b' => {
                self.i += 2;
                let v = self.num_until(b';')?;
                Ok(Value::Bool(v != 0))
            }
            b'i' => {
                self.i += 2;
                Ok(Value::from(self.num_until(b';')?))
            }
            b'd' => {
                self.i += 2;
                let f = self.float_until(b';')?;
                Ok(Value::from(f))
            }
            b's' => {
                self.i += 2;
                let len = self.count_until(b':')?;
                self.eat(b'"')?;
                let end = self.i.checked_add(len).context("string length overflow")?;
                let raw = self.b.get(self.i..end).context("string overruns input")?;
                self.i = end;
                self.eat(b'"')?;
                self.eat(b';')?;
                Ok(Value::String(String::from_utf8_lossy(raw).into_owned()))
            }
            b'a' => {
                self.i += 2;
                let n = self.count_until(b':')? as i64;
                self.eat(b'{')?;
                let mut seq: Vec<Value> = Vec::with_capacity((n as usize).min(MAX_PREALLOC));
                let mut map: Map<String, Value> = Map::new();
                let mut is_seq = true;
                for k in 0..n {
                    let key = self.value()?;
                    let val = self.value()?;
                    let idx_ok = matches!(&key, Value::Number(m) if m.as_i64() == Some(k));
                    if !idx_ok {
                        is_seq = false;
                    }
                    if is_seq {
                        seq.push(val);
                    } else {
                        let ks = match &key {
                            Value::String(s) => s.clone(),
                            Value::Number(m) => m.to_string(),
                            _ => format!("{:?}", key),
                        };
                        // preserve earlier indexed entries if mixed
                        for (j, v) in seq.drain(..).enumerate() {
                            map.insert(j.to_string(), v);
                        }
                        map.insert(ks, val);
                    }
                }
                self.eat(b'}')?;
                if is_seq {
                    Ok(Value::Array(seq))
                } else {
                    Ok(Value::Object(map))
                }
            }
            b'O' => {
                self.i += 2;
                let nlen = self.count_until(b':')?;
                self.eat(b'"')?;
                let end = self.i.checked_add(nlen).context("class name length overflow")?;
                let name = String::from_utf8_lossy(
                    self.b.get(self.i..end).context("class name overruns")?,
                )
                .into_owned();
                self.i = end;
                self.eat(b'"')?;
                self.eat(b':')?;
                let n = self.count_until(b':')? as i64;
                self.eat(b'{')?;
                let mut map: Map<String, Value> = Map::new();
                map.insert("__class".into(), Value::String(name));
                for _ in 0..n {
                    let key = self.value()?;
                    let val = self.value()?;
                    let ks = match &key {
                        Value::String(s) => s.clone(),
                        Value::Number(m) => m.to_string(),
                        _ => format!("{:?}", key),
                    };
                    // PHP encodes private/protected props as \0Class\0prop — strip
                    let ks = ks.replace('\0', "");
                    map.insert(ks, val);
                }
                self.eat(b'}')?;
                Ok(Value::Object(map))
            }
            b'r' | b'R' => {
                // reference — resolve as null
                self.i += 2;
                self.num_until(b';')?;
                Ok(Value::Null)
            }
            c => bail!("unexpected type byte '{}' at {}", c as char, self.i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_lengths_error_not_abort() {
        for bad in [b"a:-1:{".as_slice(), b"O:-1:\"X\":0:{}".as_slice(), b"s:-1:\"\"".as_slice(), b"a:2:{i:-1;i:0;".as_slice()] {
            assert!(unserialize(bad).is_err(), "{bad:?} must error");
        }
    }

    #[test]
    fn trailing_bytes_error() {
        assert!(unserialize(b"i:1;junk").is_err());
        assert!(unserialize(b"N;junk").is_err());
        assert!(unserialize(b"N").is_err(), "overrun must error");
        assert!(unserialize(b"N;").is_ok(), "exact input still parses");
        assert!(unserialize(b"N;\n").is_ok(), "trailing whitespace tolerated");
        assert!(unserialize(b"i:42; \r\n").is_ok());
    }

    #[test]
    fn huge_declared_len_errors_cleanly() {
        assert!(unserialize(b"a:999999999:{").is_err());
        assert!(unserialize(b"s:999999999:\"x".as_slice()).is_err());
    }

    #[test]
    fn happy_paths_unchanged() {
        assert_eq!(unserialize(b"i:42;").unwrap(), Value::from(42));
        assert_eq!(unserialize(b"a:2:{i:0;s:1:\"a\";i:1;s:1:\"b\";}").unwrap(),
                   serde_json::json!(["a", "b"]));
    }

    #[test]
    fn deep_nesting_bails() {
        // Nesting via values: `a:1:{i:0;<inner>}`. 600 deep must error
        // (MAX_DEPTH=512), not overflow the stack.
        let mut blob = Vec::new();
        for _ in 0..300 { blob.extend_from_slice(b"a:1:{i:0;"); }
        blob.extend_from_slice(b"i:7;");
        for _ in 0..600 { blob.push(b'}'); }
        assert!(unserialize(&blob).is_err(), "deep input must bail at MAX_DEPTH");
        // Shallow sane input still parses.
        let mut ok = Vec::new();
        for _ in 0..100 { ok.extend_from_slice(b"a:1:{i:0;"); }
        ok.extend_from_slice(b"i:7;");
        for _ in 0..100 { ok.push(b'}'); }
        assert!(unserialize(&ok).is_ok());
    }
}