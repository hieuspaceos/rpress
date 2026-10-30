//! CSS scoper — Rust port of `scripts/scope-css.py` (deleted with this
//! port; kept here verbatim in the doc comment for parity):
//!
//! ```python
//! def scope_css(css): strip comments; for each rule: at-rules pass
//! through per name (media/supports/layer recurse, keyframes/font-face
//! verbatim, import verbatim, charset/namespace dropped); style rules
//! get every comma-separated selector prefixed with
//! `.gdlr-core-pb-livemode` (html/body/:root collapse onto the scope
//! itself).
//! ```
//!
//! Used by `theme_traveltour::migrate` to emit `<name>.scoped.css` twins
//! beside per-site theme css files so the Live Mode canvas
//! (`/admin/front-css/`) renders theme rules without leaking into wp-admin.

const SCOPE: &str = ".gdlr-core-pb-livemode";

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let b = css.as_bytes();
    let mut i = 0;
    // `start` tracks the first byte not yet emitted; we push whole `&str`
    // slices so multibyte UTF-8 is copied verbatim, never byte-by-byte.
    let mut start = 0;
    while i + 1 < b.len() {
        if b[i] == b'/' && b[i + 1] == b'*' {
            out.push_str(&css[start..i]);
            match css[i + 2..].find("*/") {
                Some(j) => i += 2 + j + 2,
                // Unterminated comment: drop the tail — a comment that never
                // closes swallows the rest of the sheet anyway. Deliberate
                // change from the byte-wise baseline (which kept the tail).
                None => i = b.len(),
            }
            start = i;
        } else {
            i += 1;
        }
    }
    out.push_str(&css[start..]);
    out
}

fn scope_selector(sel: &str) -> String {
    let sel = sel.trim();
    if sel.is_empty() {
        return String::new();
    }
    let low = sel.to_lowercase();
    if low == "html" || low == ":root" {
        return SCOPE.to_string();
    }
    if low == "body" {
        return SCOPE.to_string();
    }
    if let Some(rest) = low
        .strip_prefix("body ")
        .or_else(|| low.strip_prefix("body."))
        .or_else(|| low.strip_prefix("body#"))
        .or_else(|| low.strip_prefix("body:"))
        .or_else(|| low.strip_prefix("body["))
        .or_else(|| low.strip_prefix("html "))
    {
        let cut = sel.len() - rest.len();
        return format!("{SCOPE}{}", &sel[cut - 1..]);
    }
    format!("{SCOPE} {sel}")
}

/// Parse one rule at `i`; returns (emitted text, next index).
fn scope_block(css: &str, i: usize) -> (String, usize) {
    let rest = &css[i..];
    let m = rest.find(|c| c == '{' || c == ';' || c == '}');
    let Some(j) = m.map(|j| i + j) else {
        return (css[i..].to_string(), css.len());
    };
    let head = css[i..j].trim();
    let ch = css.as_bytes()[j];
    if ch == b';' {
        return (css[i..=j].to_string(), j + 1);
    }
    if ch == b'}' {
        return (css[i..j].to_string(), j);
    }
    // ch == '{' — find the matching close.
    let mut depth = 0;
    let mut k = j;
    let bytes = css.as_bytes();
    while k < bytes.len() {
        if bytes[k] == b'{' {
            depth += 1;
        } else if bytes[k] == b'}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        k += 1;
    }
    let inner = &css[j + 1..k];
    let end = k + 1;
    if head.starts_with('@') {
        let name = head[1..]
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();
        match name.as_str() {
            "media" | "supports" | "layer" => {
                return (format!("{head}{{{}}}", scope_css(inner)), end)
            }
            "font-face" | "page" | "property" | "viewport" | "keyframes"
            | "-webkit-keyframes" | "-moz-keyframes" | "import" => {
                return (css[i..end].to_string(), end)
            }
            "charset" | "namespace" => return (String::new(), end),
            _ => return (css[i..end].to_string(), end),
        }
    }
    let sels: Vec<String> = head.split(',').map(scope_selector).collect();
    (format!("{}{{{inner}}}", sels.join(",\n")), end)
}

pub fn scope_css(css: &str) -> String {
    let css = strip_comments(css);
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < css.len() {
        if css[i..].find(|c| c == '{' || c == ';' || c == '}').is_none() {
            let tail = css[i..].trim();
            if !tail.is_empty() {
                out.push(tail.to_string());
            }
            break;
        }
        let (text, next) = scope_block(&css, i);
        if !text.trim().is_empty() {
            out.push(text);
        }
        i = next;
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_plain_and_body_selectors() {
        let out = scope_css("a{color:red} body.pad{margin:0} :root{--x:1}");
        assert!(out.contains(".gdlr-core-pb-livemode a{color:red}"), "{out}");
        assert!(out.contains(".gdlr-core-pb-livemode.pad{margin:0}"), "{out}");
        assert!(out.contains(".gdlr-core-pb-livemode{--x:1}"), "{out}");
    }

    #[test]
    fn media_recurse_and_keyframes_verbatim() {
        let out = scope_css(
            "@media (max-width:1px){.x{y:1}} @keyframes spin{from{a:0}to{a:1}} @import 'x';",
        );
        assert!(out.contains(".gdlr-core-pb-livemode .x"), "{out}");
        assert!(out.contains("from{a:0}"), "{out}");
        assert!(out.contains("@import"), "{out}"); // ';' rules pass verbatim (parity with scope-css.py)
    }

    #[test]
    fn non_ascii_roundtrips_byte_identical() {
        // Multibyte UTF-8 in declarations must not be mangled into
        // Latin-1 mojibake by byte-wise copying.
        let css = ".t::after{content:\"héllo — wörld ☕\"} /* kommentar äöü */ .u{background:url('café.png')}";
        let out = scope_css(css);
        assert!(out.contains("héllo — wörld ☕"), "{out}");
        assert!(out.contains("café.png"), "{out}");
        // é widened byte-wise would appear as "Ã©"; none may be present.
        assert!(!out.contains("Ã©"), "{out}");
        // Byte-widening turns each `é` into the two chars Ã and ©; the
        // two `é`s (héllo, café) must survive intact.
        assert_eq!(out.matches('é').count(), 2, "{out}");
    }

    #[test]
    fn import_rule_passes_verbatim() {
        let css = "@import 'https://fonts.googleapis.com/css2?family=Inter:wght@400&display=swap'; .a{x:1}";
        let out = scope_css(css);
        assert!(
            out.contains(
                "@import 'https://fonts.googleapis.com/css2?family=Inter:wght@400&display=swap';"
            ),
            "{out}"
        );
        // Surrounding rules are still scoped — @import is untouched.
        assert!(out.contains(".gdlr-core-pb-livemode .a{x:1}"), "{out}");
    }
}
