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
