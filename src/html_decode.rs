//! `html_decode` — extracted from `admin::gdlr_pb` (phase-07 T30) so the
//! public render path keeps working after the port is deleted.
//!
//! PHP option defs pass `esc_html__()` output — strings arrive
//! pre-escaped (`&amp;`, `&eacute;`…). Decode before re-escaping for an
//! HTML context so labels show `&` not `&amp;`. Only the named entities
//! GoodLayers emits are in the table; anything unknown passes through
//! verbatim.

pub(crate) fn html_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let semi = tail.find(';').filter(|&j| j < 10);
        match semi {
            Some(j) => {
                let ent = &tail[..=j];
                let dec = match ent {
                    "&amp;" => "&", "&lt;" => "<", "&gt;" => ">", "&quot;" => "\"",
                    "&#039;" | "&#39;" | "&apos;" => "'", "&nbsp;" => " ",
                    "&eacute;" => "é", "&egrave;" => "è", "&agrave;" => "à",
                    "&ecirc;" => "ê", "&ocirc;" => "ô", "&ucirc;" => "û",
                    "&iuml;" => "ï", "&ccedil;" => "ç", "&ndash;" => "–",
                    "&mdash;" => "—", "&hellip;" => "…", "&middot;" => "·",
                    _ => { out.push_str(ent); rest = &tail[j + 1..]; continue; }
                };
                out.push_str(dec);
                rest = &tail[j + 1..];
            }
            None => { out.push('&'); rest = &tail[1..]; }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_named_and_numeric() {
        assert_eq!(super::html_decode("caf&eacute; &amp; &lt;x&gt; &#039;q&#039;"), "café & <x> 'q'");
        assert_eq!(super::html_decode("a&unknown;b&"), "a&unknown;b&");
        assert_eq!(super::html_decode("&nbsp;x"), " x");
    }
}
