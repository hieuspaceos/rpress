//! HTML → markdown + text-extraction helpers for the `.md` siblings
//! (moved out of render.rs — pure move, no behavior change).

#[allow(unused_imports)]
use super::*;

/// Strip HTML tags, collapse whitespace; entities left as-is (WP excerpt
/// behaves the same for our purposes since &amp; etc. are rare here).
pub(crate) fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Auto description fallback (RankMath when no seo_desc/excerpt): first
/// paragraph after the first heading, else the first text block; capped at
/// 30 words (the observed RankMath excerpt length).
pub(crate) fn auto_desc(body_html: &str) -> String {
    let h_end = body_html.find("</h").map(|i| {
        body_html[i..]
            .find('>')
            .map(|o| i + o + 1)
            .unwrap_or(i)
    });
    let hay = h_end.map(|i| &body_html[i..]).unwrap_or(body_html);
    // first <p>…</p> if present, else the first non-empty line
    let text = if let Some(ps) = hay.find("<p") {
        let rest = &hay[ps..];
        let start = rest.find('>').map(|o| o + 1).unwrap_or(0);
        let end = rest[start..].find("</p").map(|o| start + o).unwrap_or(rest.len());
        strip_tags(&rest[start..end])
    } else {
        let line = hay.split('\n').find(|l| !strip_tags(l).is_empty()).unwrap_or("");
        strip_tags(line)
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() > 30 {
        words[..30].join(" ")
    } else {
        words.join(" ")
    }
}

/// Rough word count of rendered content for the reading-time label.
pub(crate) fn word_count(html: &str) -> usize {
    strip_tags(html).split_whitespace().count()
}
/// Reduced HTML→markdown for the `.md` content-negotiation surface
/// (`/{slug}/index.md` + Accept negotiation in `public_page`). Covers the
/// block shapes the post body produces (headings, paragraphs, lists,
/// quotes, tables, links, images); anything else degrades to plain text.
/// This is a one-pass converter, not an HTML parser — malformed input can
/// only leak stray tags, never panic.
pub fn html_to_markdown(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut buf = String::new();
    fn take_until<'a>(hay: &'a str, needle: &str) -> (String, &'a str) {
        match hay.find(needle) {
            Some(i) => (hay[..i].to_string(), &hay[i + needle.len()..]),
            None => (hay.to_string(), ""),
        }
    }
    let mut cur_text = String::new();
    macro_rules! flush_text {
        () => {
            let t = inline_md(&cur_text);
            if !t.is_empty() {
                buf.push_str(&t);
                buf.push_str("\n\n");
            }
            cur_text.clear();
        };
    }
    while !rest.is_empty() {
        if let Some(i) = rest.find('<') {
            cur_text.push_str(&rest[..i]);
            rest = &rest[i..];
            // tag name
            let name_end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(rest.len());
            let tag = rest[1..name_end.min(rest.len())].to_lowercase();
            // attrs sit between name_end and the tag's '>' — skip to it
            // before grabbing the inner span so they can't leak into text.
            let gt_rel = rest[name_end..].find('>').map(|i| name_end + i + 1).unwrap_or(rest.len());
            match tag.as_str() {
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    flush_text!();
                    let level: usize = tag[1..].parse().unwrap_or(2);
                    let (inner, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                    let hashes = "#".repeat(level.min(6));
                    buf.push_str(&format!("{hashes} {}\n\n", inline_md(&inner)));
                }
                "p" => {
                    let (inner, r) = take_until(&rest[gt_rel..], "</p>");
                    rest = r;
                    cur_text.push_str(&inner);
                    flush_text!();
                }
                "br" => {
                    cur_text.push(' ');
                    let (_, r) = take_until(rest, ">");
                    rest = r;
                }
                "ul" | "ol" => {
                    flush_text!();
                    let ordered = tag == "ol";
                    let (inner, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                    let mut n = 0usize;
                    let mut items = inner.as_str();
                    while let Some(li) = items.find("<li") {
                        let (body, r2) = take_until(&items[li..], "</li>");
                        let body = body.split_once('>').map(|(_, b)| b).unwrap_or("");
                        n += 1;
                        let marker = if ordered { format!("{n}.") } else { "-".into() };
                        buf.push_str(&format!("{marker} {}\n", inline_md(body)));
                        items = r2;
                    }
                    buf.push('\n');
                }
                "blockquote" => {
                    flush_text!();
                    let (inner, r) = take_until(&rest[gt_rel..], "</blockquote>");
                    rest = r;
                    for line in html_to_markdown(&inner).lines() {
                        buf.push_str(&format!("> {line}\n"));
                    }
                    buf.push('\n');
                }
                "table" => {
                    flush_text!();
                    let (inner, r) = take_until(&rest[gt_rel..], "</table>");
                    rest = r;
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    let mut rrest = inner.as_str();
                    while let Some(tr) = rrest.find("<tr") {
                        let (row_html, r2) = take_until(&rrest[tr..], "</tr>");
                        rrest = r2;
                        let mut cells = Vec::new();
                        let mut crest = row_html.as_str();
                        while let Some(td) = crest.find("<td").into_iter().chain(crest.find("<th")).min() {
                            let (cell, r3) = take_until(&crest[td..], if crest[td..].starts_with("<th") { "</th>" } else { "</td>" });
                            let cell = cell.split_once('>').map(|(_, c)| c).unwrap_or("");
                            cells.push(inline_md(cell).replace('|', "\\|").replace('\n', " "));
                            crest = r3;
                        }
                        if !cells.is_empty() { rows.push(cells); }
                    }
                    if !rows.is_empty() {
                        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
                        for (i, row) in rows.iter().enumerate() {
                            let mut padded = row.clone();
                            padded.resize(w, String::new());
                            buf.push_str(&format!("| {} |\n", padded.join(" | ")));
                            if i == 0 {
                                buf.push_str(&format!("|{}|\n", "|".repeat(w + 1).replace("||", "| --- ")));
                            }
                        }
                        buf.push('\n');
                    }
                }
                "figure" | "div" | "section" | "article" | "header" | "footer" | "main" | "aside" => {
                    let (inner, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                    cur_text.push_str(&inner);
                    flush_text!();
                }
                "script" | "style" | "noscript" | "iframe" | "form" | "button" | "select" | "input" | "textarea" => {
                    // skip entire element — not content
                    let (_, r) = take_until(&rest[gt_rel..], &format!("</{tag}>"));
                    rest = r;
                }
                "a" => {
                    // keep inner text; inline_md handles <a> if we left it
                    let gt = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
                    rest = &rest[gt..];
                }
                "img" => {
                    let (attrs, r) = take_until(rest, ">");
                    rest = r;
                    if let Some(alt) = extract_attr(&attrs, "alt") {
                        if !alt.is_empty() { cur_text.push_str(&alt); }
                    }
                }
                _ => {
                    // unknown/inline tag → drop the tag only
                    let gt = rest.find('>').map(|i| i + 1).unwrap_or(rest.len());
                    rest = &rest[gt..];
                }
            }
        } else {
            cur_text.push_str(rest);
            break;
        }
    }
    flush_text!();
    out.push_str(&buf);
    // collapse 3+ blank lines
    let mut collapsed = String::with_capacity(out.len());
    let mut blank = 0;
    for line in out.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank <= 1 { collapsed.push('\n'); }
        } else {
            blank = 0;
            collapsed.push_str(line);
            collapsed.push('\n');
        }
    }
    collapsed.trim().to_string()
}

/// Inline-level conversion inside a block: <strong>/<b> → **, <em>/<i> → *,
/// <a href>text</a> → [text](href), <img> → ![alt](src), <code> → `code`,
/// entities decoded minimally.
fn inline_md(s: &str) -> String {
    let mut t = s.to_string();
    // links before generic tag-strip so we keep the href
    while let Some(a) = t.find("<a ") {
        let gt = t[a..].find('>').map(|i| a + i + 1).unwrap_or(t.len());
        let attrs = t[a + 3..gt.saturating_sub(1)].to_string();
        let (inner, rest) = take_once(&t[gt..], "</a>");
        let href = extract_attr(&attrs, "href").unwrap_or_default();
        let label = strip_tags(&inner);
        t = format!("{}{}", &t[..a], if href.is_empty() { label } else { format!("[{label}]({href})") }) + rest;
    }
    while let Some(i) = t.find("<img") {
        let (attrs, rest) = take_once(&t[i..], ">");
        let alt = extract_attr(&attrs, "alt").unwrap_or_default();
        let src = extract_attr(&attrs, "src").unwrap_or_default();
        t = format!("{}{}{}", &t[..i], if src.is_empty() { alt } else { format!("![{alt}]({src})") }, rest);
    }
    for (tag, rep) in [("strong", "**"), ("b", "**"), ("em", "*"), ("i", "*"), ("code", "`"), ("mark", "==")] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(a) = t.find(&open) {
            let gt = t[a..].find('>').map(|i| a + i + 1).unwrap_or(t.len());
            let (inner, rest) = take_once(&t[gt..], &close);
            let inner_txt = strip_tags(inner);
            t = format!("{}{}{}{}", &t[..a], rep, inner_txt, rep) + rest;
        }
    }
    // drop residual tags, decode common entities
    strip_tags(&t)
        .replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">")
        .replace("&quot;", "\"").replace("&#039;", "'").replace("&nbsp;", " ")
        .replace("&eacute;", "é").replace("&egrave;", "è").replace("&agrave;", "à")
        .replace("&ccedil;", "ç").replace("&ocirc;", "ô").replace("&icirc;", "î")
        .replace("&ecirc;", "ê").replace("&ucirc;", "û").replace("&euml;", "ë")
        .replace("&iuml;", "ï").replace("&ugrave;", "ù")
        .split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Take until `needle`, returning (before, after).
fn take_once<'a>(hay: &'a str, needle: &str) -> (&'a str, &'a str) {
    match hay.find(needle) {
        Some(i) => (&hay[..i], &hay[i + needle.len()..]),
        None => (hay, ""),
    }
}

/// Extract an HTML attribute value from a tag-attr fragment.
fn extract_attr(attrs: &str, name: &str) -> Option<String> {
    let pat = format!("{name}=");
    let i = attrs.find(&pat)? + pat.len();
    let q = attrs.as_bytes().get(i).copied()?;
    if q != b'"' && q != b'\'' { return None; }
    let rest = &attrs[i + 1..];
    rest.find(q as char).map(|e| rest[..e].to_string())
}

/// Markdown page for one published post (`.md` sibling of the HTML page).
/// Front-matter + converted body; links/images keep absolute site URLs.
pub fn post_markdown(conn: &Connection, slug: &str, lang: &str, kind: &str, site_url: &str, tour_base: &str) -> Option<String> {
    post_markdown_id(conn, slug, lang, kind, site_url, tour_base).map(|(_id, md)| md)
}

/// `post_markdown` plus the resolved post id — the public_page canonical
/// gate needs the id to compare request path vs canonical permalink (a
/// `.md` on an alias URL 301s to the canonical `.md` instead of caching
/// under the alias).
pub(crate) fn post_markdown_id(conn: &Connection, slug: &str, lang: &str, kind: &str, site_url: &str, tour_base: &str) -> Option<(i64, String)> {
    let (id, title, body_html, excerpt, published_at, kind, slug, body_json): (i64, String, String, String, String, String, String, String) = conn
        .query_row(sql!("render__select_markdown_id"), rusqlite::params![slug, lang, kind], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?))
        })
        .ok()?;
    // tours carry the page-builder tree in body_json (body_html is NULL):
    // walk it and pull every "content"/"text" field so the .md still has
    // the real copy, not an empty body.
    let body_html = if body_html.trim().is_empty() {
        body_json_text(&body_json)
    } else {
        body_html
    };
    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!("title: \"{}\"\n", title.replace('"', "\\\"")));
    md.push_str(&format!("url: {}\n", page_url(conn, site_url.trim_end_matches('/'), lang, &kind, &slug, tour_base)));
    if !published_at.is_empty() { md.push_str(&format!("date: {}\n", iso8601(&published_at))); }
    md.push_str(&format!("type: {kind}\n---\n\n# {title}\n\n"));
    if !excerpt.is_empty() {
        md.push_str(&format!("> {}\n\n", excerpt.trim()));
    }
    md.push_str(&html_to_markdown(&body_html));
    md.push('\n');
    Some((id, md))
}

/// Collect every text-ish leaf out of a page-builder body_json tree —
/// element "content" fields hold HTML fragments; other string leaves are
/// titles/labels. Returns concatenated HTML for html_to_markdown.
fn body_json_text(json: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut out = String::new();
    fn walk(v: &serde_json::Value, out: &mut String) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, val) in m {
                    if matches!(k.as_str(), "content" | "text" | "excerpt" | "description") {
                        if let Some(s) = val.as_str() {
                            out.push_str(s);
                            out.push_str("\n\n");
                        }
                    }
                    walk(val, out);
                }
            }
            serde_json::Value::Array(a) => {
                for x in a {
                    walk(x, out);
                }
            }
            _ => {}
        }
    }
    walk(&v, &mut out);
    out
}

/// WP `_n` reading time: <200 words → "Moins d’une minute", else N minutes
/// at 200 wpm (matches the reference post: 2033 words → "10 minutes").
pub(crate) fn reading_time(words: usize, lang: &str) -> String {
    if words < 200 {
        return crate::lang::t(lang, "under_minute").into();
    }
    let n = ((words as f64) / 200.0).round().max(1.0) as u64;
    format!("{n} {}", crate::lang::t(lang, "minutes"))
}
