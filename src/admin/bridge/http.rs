//! Minimal HTTP/1.1 client for bridge proxying — one-shot
//! connection-close requests over `tokio::net::TcpStream`. No deps
//! (reqwest/hyper-client not in the tree); requests are buffered, the
//! bridge is localhost-only, and body sizes are plugin-asset bounded.

use super::BridgeResponse;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Send one request; returns parsed response. Sync-blocking — callers
/// wrap in `tokio::task::spawn_blocking` (bridge calls are rare admin
/// requests, not hot path).
pub(crate) fn request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<BridgeResponse> {
    // Request-line hygiene: `path` arrives percent-decoded from axum — a
    // caller-supplied CRLF would inject headers, `..` escapes the php -S
    // docroot, `?`/`#`/`:`/`//` smuggle query/absolute-URI shapes the proxy
    // never legitimately emits. Reject at the transport chokepoint so every
    // caller is covered.
    anyhow::ensure!(
        !path.bytes().any(|b| b < 0x20 || b == b' ' || b == 0x7f)
            && !path.split('/').any(|s| s == "..")
            && !path.contains('?')
            && !path.contains('#')
            && !path.contains(':')
            && !path.contains('\\')
            && !path.starts_with("//"),
        "bridge path rejected: {path:?}"
    );
    let mut s = TcpStream::connect(("127.0.0.1", port))
        .with_context(|| format!("bridge connect 127.0.0.1:{port}"))?;
    s.set_read_timeout(Some(Duration::from_secs(120)))?;
    s.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut req = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    for (k, v) in headers {
        let kl = k.to_ascii_lowercase();
        // Hop-by-hop + framing headers never cross the proxy; we re-emit
        // Content-Length below (forwarding the client's value would be
        // correct too — the body is unmodified — but recomputing is the
        // one code path that can't drift).
        // Credentials never cross either: a live `rpress_session` cookie,
        // app-password `Authorization`, or a bearer `X-Forwarded-*` chain
        // handed to vendored plugin code is exfiltratable via `$_SERVER`
        // (audit M1). The bridge's auth is `X-Rpress-Bridge` only.
        if matches!(kl.as_str(), "host" | "connection" | "content-length" | "transfer-encoding" | "cookie" | "expect" | "authorization" | "proxy-authorization" | "x-rpress-bridge")
            || kl.starts_with("x-forwarded-")
            || kl.starts_with("x-real-")
        {
            continue;
        }
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    req.push_str("\r\n");
    s.write_all(req.as_bytes())?;
    if !body.is_empty() {
        s.write_all(body)?;
    }
    s.flush()?;

    let mut raw = Vec::new();
    s.read_to_end(&mut raw)?;
    parse(&raw)
}

/// Split + parse a buffered HTTP/1.1 response (connection-close means the
/// body is "everything after the headers", chunked or not).
fn parse(raw: &[u8]) -> Result<BridgeResponse> {
    let pos = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .context("bridge response: no header terminator")?;
    let head = std::str::from_utf8(&raw[..pos]).context("bridge response headers not utf8")?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().context("empty response")?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .context("bad status line")?;
    let mut headers = Vec::new();
    let mut chunked = false;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            if k.eq_ignore_ascii_case("transfer-encoding") && v.eq_ignore_ascii_case("chunked") {
                chunked = true;
            }
            headers.push((k, v));
        }
    }
    let body_raw = &raw[pos + 4..];
    let body = if chunked { dechunk(body_raw)? } else { body_raw.to_vec() };
    Ok(BridgeResponse { status, headers, body })
}

fn dechunk(mut b: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let eol = b.windows(2).position(|w| w == b"\r\n").context("chunk: no size line")?;
        let sz = std::str::from_utf8(&b[..eol])
            .ok()
            .and_then(|s| usize::from_str_radix(s.trim(), 16).ok())
            .context("chunk: bad size")?;
        b = &b[eol + 2..];
        if sz == 0 {
            break;
        }
        anyhow::ensure!(b.len() >= sz + 2, "chunk: truncated");
        out.extend_from_slice(&b[..sz]);
        b = &b[sz + 2..];
    }
    Ok(out)
}

/// Probe endpoint — GET `/` with a tiny timeout.
pub(crate) fn probe(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port))
        .map(|mut s| {
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let _ = s.set_write_timeout(Some(Duration::from_secs(2)));
            let _ = s.write_all(b"GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n");
            let mut buf = [0u8; 256];
            matches!(s.read(&mut buf), Ok(n) if n > 0)
        })
        .unwrap_or(false)
}

/// Identity probe — GET `/__rpress_ident` with the spawn-time bearer.
/// `true` only when the endpoint answers 204 with our header, which no
/// foreign listener (or a bridge holding a *different* token) can fake:
/// the mu-plugin 403s unknown tokens before this route ever runs.
pub(crate) fn probe_identity(port: u16, token: &str) -> bool {
    TcpStream::connect(("127.0.0.1", port))
        .map(|mut s| {
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let _ = s.set_write_timeout(Some(Duration::from_secs(2)));
            let req = format!(
                "GET /__rpress_ident HTTP/1.1\r\nHost: x\r\nConnection: close\r\nX-Rpress-Bridge: {token}\r\n\r\n"
            );
            let _ = s.write_all(req.as_bytes());
            let mut buf = [0u8; 512];
            match s.read(&mut buf) {
                Ok(n) if n > 0 => std::str::from_utf8(&buf[..n])
                    .map(|h| h.starts_with("HTTP/1.1 204") || h.starts_with("HTTP/1.0 204"))
                    .unwrap_or(false),
                _ => false,
            }
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dechunk_basic() {
        let raw = b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        assert_eq!(dechunk(raw).unwrap(), b"Wikipedia");
    }

    #[test]
    fn parse_simple() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nX-A: b\r\n\r\nhello";
        let r = parse(raw).unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, b"hello");
        assert!(r.headers.iter().any(|(k, _)| k == "X-A"));
    }
}
