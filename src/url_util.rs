//! URL helpers shared by menu resolution and plugin asset rewriting.

/// Strip `url`'s scheme+host when the host is one of the site's own
/// origins — `site_url` itself plus any alias hosts (`url_aliases`
/// setting, e.g. `test.aucoeurvietnam.com`, `reise.test.…`). Own-origin
/// URLs come back as `{site_url}`-absolute (single canonical form so
/// post-render matchers like `apply_navx` keep working). Foreign hosts
/// pass through unchanged — external links must never become
/// site-relative paths.
///
/// `aliases` is a comma-separated host list (may include scheme — the
/// host part is what matters).
pub fn strip_origin(url: &str, site_url: &str, aliases: &str) -> String {
    let Some(rest) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    else {
        return url.to_string(); // already relative or protocol-relative
    };
    let host_end = rest.find('/').unwrap_or(rest.len());
    let host = &rest[..host_end];
    let site_host = site_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or("");
    let alias_hit = aliases
        .split(',')
        .map(|a| {
            a.trim()
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .trim_end_matches('/')
        })
        .any(|a| !a.is_empty() && a == host);
    if host != site_host && !alias_hit {
        return url.to_string();
    }
    // Re-anchor on the live site_url — menu/nav matchers key absolute.
    let path = &rest[host_end..];
    format!("{}{}", site_url.trim_end_matches('/'), if path.is_empty() { "/" } else { path })
}

/// Scheme gate for user-stored link URLs (menu items, widget links).
/// WP's `esc_url` equivalent: `http:`, `https:`, `mailto:`, `tel:` and
/// site-relative (`/`, `#`, `?`) shapes pass; `javascript:`, `data:`,
/// `vbscript:` and anything else exotic is rejected. Protocol-relative
/// `//host` is allowed (same-class as https to a foreign host).
/// Whitespace/control chars are stripped before judging — `java\tscript:`
/// is a real bypass otherwise.
pub fn safe_href(url: &str) -> bool {
    let u: String = url.chars().filter(|c| !c.is_whitespace() && *c != '\u{0}').collect();
    let u = u.trim();
    if u.is_empty() {
        return true; // container items render without href upstream
    }
    if u.starts_with('/') || u.starts_with('#') || u.starts_with('?') {
        return !u.starts_with("//") || u[2..].split('/').next().is_some_and(|h| !h.is_empty());
    }
    match u.find(':') {
        Some(i) => matches!(
            u[..i].to_ascii_lowercase().as_str(),
            "http" | "https" | "mailto" | "tel"
        ),
        // no `scheme:` — relative path like `foo/bar`, allowed
        None => true,
    }
}

/// Render-time companion: anything `safe_href` rejects becomes the
/// inert `#` — never emit a stored hostile URL even if an older DB row
/// or import carried one (audit M2).
pub fn safe_href_or_hash(url: &str) -> String {
    if safe_href(url) { url.to_string() } else { "#".to_string() }
}

/// Sanitize a menu label that may intentionally carry an `<img>` (the
/// image-menu items rewritten at import). Rules: text is HTML-escaped;
/// only `<img>` tags survive, rebuilt from an attr allowlist
/// (src/alt/class/width/height/srcset/loading/decoding/title) with `src`
/// passing `safe_href` OR `data:image/`; every other tag is dropped and
/// surrounding text escaped. Returns HTML ready for raw insertion.
pub fn sanitize_menu_label(raw: &str) -> String {
    // Stored labels already carry WP's entity-escaping (`Trek &amp;
    // randonnée` lives in menu_items.label verbatim — WP stores
    // escaped). Decode entities first, then re-escape, so legit stored
    // entities round-trip byte-identical instead of double-escaping
    // (`&amp;amp;`) while truly raw text still gets escaped.
    fn esc_text(s: &str) -> String {
        crate::html_decode::html_decode(s)
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
    fn esc_attr_v(s: &str) -> String {
        crate::html_decode::html_decode(s)
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
    fn img_tag(inner: &str) -> String {
        // Attr scanner over `inner` (contents after `<`, before `>`).
        let rest = inner.trim_end_matches('/');
        let mut attrs = String::new();
        let bytes = rest.as_bytes();
        let mut i = 0;
        // skip the `img` name itself
        while i < bytes.len() && !(bytes[i] as char).is_whitespace() { i += 1; }
        while i < bytes.len() {
            while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
            if i >= bytes.len() { break; }
            let start = i;
            while i < bytes.len() && bytes[i] != b'=' && !(bytes[i] as char).is_whitespace() { i += 1; }
            let name = &rest[start..i];
            while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
            if i >= bytes.len() || bytes[i] != b'=' { continue; }
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() { i += 1; }
            if i >= bytes.len() { break; }
            let (val, adv) = if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                let s = &rest[i + 1..];
                match s.find(q as char) {
                    Some(e) => (s[..e].to_string(), e + 2),
                    None => (s.to_string(), s.len() + 1),
                }
            } else {
                let s = &rest[i..];
                let e = s.find(char::is_whitespace).unwrap_or(s.len());
                (s[..e].to_string(), e)
            };
            i += adv;
            let lname = name.to_ascii_lowercase();
            let ok = matches!(
                lname.as_str(),
                "src" | "alt" | "class" | "width" | "height" | "srcset" | "loading" | "decoding" | "title"
            );
            if !ok { continue; }
            if lname == "src" && !(safe_href(&val) || val.starts_with("data:image/")) {
                continue; // hostile src → drop the attr; img without src is inert
            }
            if lname == "srcset" && !val.split(',').all(|c| {
                let u = c.trim().split_whitespace().next().unwrap_or("");
                u.is_empty() || safe_href(u) || u.starts_with("data:image/")
            }) {
                continue;
            }
            attrs.push_str(&format!(" {}=\"{}\"", lname, esc_attr_v(&val)));
        }
        format!("<img{attrs}>")
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(i) = rest.find('<') {
        out.push_str(&esc_text(&rest[..i]));
        rest = &rest[i + 1..];
        match rest.find('>') {
            Some(e) => {
                let tag = &rest[..e];
                let tl = tag.trim_start().to_ascii_lowercase();
                if tl.strip_prefix("img").is_some_and(|r| r.is_empty() || r.starts_with(|c: char| c.is_whitespace() || c == '/' || c == '=')) && !tl.starts_with('/') {
                    out.push_str(&img_tag(tag));
                }
                // non-img tag: dropped entirely (text outside survives escaped)
                rest = &rest[e + 1..];
            }
            None => {
                out.push_str(&esc_text(&format!("<{rest}")));
                rest = "";
            }
        }
    }
    out.push_str(&esc_text(rest));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    const SITE: &str = "http://localhost:8414";
    const ORIGIN: &str = "imagetravel.vn"; // imported settings.site_url host

    #[test]
    fn own_origin_reanchors() {
        assert_eq!(
            strip_origin("https://imagetravel.vn/lien-he/", SITE, ORIGIN),
            "http://localhost:8414/lien-he/"
        );
        // served host itself also counts as own
        assert_eq!(
            strip_origin("http://localhost:8414/x/", SITE, ORIGIN),
            "http://localhost:8414/x/"
        );
    }
    #[test]
    fn alias_host_reanchors() {
        assert_eq!(
            strip_origin("https://test.aucoeurvietnam.com/x/", SITE, "imagetravel.vn,test.aucoeurvietnam.com"),
            "http://localhost:8414/x/"
        );
    }
    #[test]
    fn foreign_host_untouched() {
        let u = "https://facebook.com/page";
        assert_eq!(strip_origin(u, SITE, ORIGIN), u);
    }
    #[test]
    fn relative_untouched() {
        assert_eq!(strip_origin("/local/path", SITE, ""), "/local/path");
    }
    #[test]
    fn root_url() {
        assert_eq!(strip_origin("https://imagetravel.vn", SITE, ORIGIN), "http://localhost:8414/");
    }
}

/// Re-anchor every own-origin `href="http(s)://host/…"` occurrence inside
/// an HTML fragment on `site_url` (top-bar/copyright payloads carry the
/// origin host verbatim). Attribute-delimited `href=`/`src=` only —
/// free-text URLs stay untouched.
pub fn rewrite_own_origin(html: &str, site_url: &str, aliases: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find(['"', '\''].as_ref()) {
        // find quoted attr value starting with http
        let (q, after_q) = (&rest[i..i + 1], &rest[i + 1..]);
        if !(after_q.starts_with("http://") || after_q.starts_with("https://")) {
            out.push_str(&rest[..i + 1]);
            rest = after_q;
            continue;
        }
        let end = after_q.find(q).unwrap_or(after_q.len());
        let url = &after_q[..end];
        out.push_str(&rest[..i + 1]);
        out.push_str(&strip_origin(url, site_url, aliases));
        rest = &after_q[end..];
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod href_tests {
    use super::{safe_href, safe_href_or_hash, sanitize_menu_label};

    #[test]
    fn href_schemes() {
        assert!(safe_href("https://x.com/a"));
        assert!(safe_href("http://x.com"));
        assert!(safe_href("/local/path"));
        assert!(safe_href("#frag"));
        assert!(safe_href("?q=1"));
        assert!(safe_href("mailto:a@b.c"));
        assert!(safe_href("tel:+123"));
        assert!(safe_href(""));        // container
        assert!(safe_href("//cdn.com/x")); // protocol-relative ok
        assert!(safe_href("relative/path"));
        assert!(!safe_href("javascript:alert(1)"));
        assert!(!safe_href("JavaScript:alert(1)"));
        assert!(!safe_href("java\tscript:alert(1)"));
        assert!(!safe_href("java script:alert(1)"));
        assert!(!safe_href("data:text/html,<script>"));
        assert!(!safe_href("vbscript:x"));
        assert!(!safe_href("file:///etc/passwd"));
        assert_eq!(safe_href_or_hash("javascript:x"), "#");
        assert_eq!(safe_href_or_hash("/ok"), "/ok");
    }

    #[test]
    fn label_sanitize() {
        // plain text passes through
        assert_eq!(sanitize_menu_label("Tours"), "Tours");
        // img survives with allowed attrs
        assert_eq!(
            sanitize_menu_label(r#"<img src="/f.svg" alt="FR" width="20">"#),
            r#"<img src="/f.svg" alt="FR" width="20">"#
        );
        // hostile attrs stripped, safe ones kept
        assert_eq!(
            sanitize_menu_label(r#"<img src="javascript:x" onerror="alert(1)" alt="x">"#),
            r#"<img alt="x">"#
        );
        // non-img tags dropped, text escaped
        assert_eq!(
            sanitize_menu_label("<script>alert(1)</script>Hi <b>bold</b>"),
            "alert(1)Hi bold"
        );
        // unclosed/escaped fragments stay literal
        assert_eq!(sanitize_menu_label("a < b & c"), "a &lt; b &amp; c");
        // quote injection in attr value neutralized
        assert_eq!(
            sanitize_menu_label(r#"<img alt='" onmouseover="x'>"#),
            r#"<img alt="&quot; onmouseover=&quot;x">"#
        );
    }
}
