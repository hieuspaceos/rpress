//! Minimal PHP `unserialize` → serde_json::Value.
//! Supports N;, b:, i:, d:, s:, a:, O: (objects → {"__class": n, ...props}).
//! References (r:/R:) resolve as null — WP payloads don't rely on them.

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};

pub fn unserialize(input: &[u8]) -> Result<Value> {
    let mut p = Parser { b: input, i: 0 };
    let v = p.value()?;
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
                let len = self.num_until(b':')? as usize;
                self.eat(b'"')?;
                let raw = self.b.get(self.i..self.i + len).context("string overruns input")?;
                self.i += len;
                self.eat(b'"')?;
                self.eat(b';')?;
                Ok(Value::String(String::from_utf8_lossy(raw).into_owned()))
            }
            b'a' => {
                self.i += 2;
                let n = self.num_until(b':')?;
                self.eat(b'{')?;
                let mut seq: Vec<Value> = Vec::with_capacity(n as usize);
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
                let nlen = self.num_until(b':')? as usize;
                self.eat(b'"')?;
                let name = String::from_utf8_lossy(
                    self.b.get(self.i..self.i + nlen).context("class name overruns")?,
                )
                .into_owned();
                self.i += nlen;
                self.eat(b'"')?;
                self.eat(b':')?;
                let n = self.num_until(b':')?;
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
