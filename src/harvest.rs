//! Origin-page harvesting shared by generate (fetch-at-generate for
//! `harvest_posts` elementor ids) and import (fetch-at-import for spec
//! `harvest = true` kinds → `harvested_pages`).
//!
//! The fetched page is the origin's own rendered output — fidelity by
//! construction. Origin URLs are rewritten to a placeholder target:
//! callers pass the local `site_url` (generate) or `{{ site_url }}`
//! (import-time storage, resolved by `rewrite_origin` at render).

use anyhow::{Context, Result};
use std::process::Command;

/// Fetch `origin + path` and rewrite every absolute origin URL to
/// `target` (the caller's site_url or the `{{ site_url }}` placeholder).
/// Also rewrites CF7 form actions so harvested enquiry forms post to
/// `/form/enquiry` (the intake endpoint) instead of a URL rpress never
/// serves — a visibly broken form is worse than a mapped one.
pub(crate) fn page(origin: &str, path: &str, target: &str) -> Result<String> {
    let base = origin.trim_end_matches('/');
    // Scheme-less stored origins need a scheme to fetch; default https.
    let base = if base.contains("://") {
        base.to_string()
    } else {
        format!("https://{base}")
    };
    let base = base.as_str();
    let url = format!("{base}/{}", path.trim_start_matches('/'));
    let o = Command::new("curl")
        .args(["-sfL", "--max-time", "30", "-A", "rpress-harvest/1.0", &url])
        .output()
        .with_context(|| format!("curl {url}"))?;
    if !o.status.success() {
        anyhow::bail!("{url} → http {}", o.status);
    }
    let mut html = String::from_utf8_lossy(&o.stdout).into_owned();
    if html.len() < 500 {
        anyhow::bail!("{url} too short ({}b) — refusing", html.len());
    }
    let local = target.trim_end_matches('/');
    html = html.replace(base, local);
    html = html.replace(&base.replace('/', "\\/"), &local.replace('/', "\\/"));
    html = rewrite_cf7_actions(&html, local, path);
    Ok(html)
}

/// CF7 renders `<form action="{page_url}#wpcf7-f{id}-o{n}">`; after the
/// origin→local rewrite that still lands on a static page URL, so a
/// submit would POST to a 404. Point CF7 forms at the `/form/enquiry`
/// intake endpoint instead and preserve the return path + post slug as
/// hidden fields (same convention as `babe_enquiry.html`). A form is
/// "CF7" when its tag or wrapper carries the wpcf7 marker AND the action
/// fragment does — a deliberate `#wpcf7` anchor check keeps non-CF7
/// forms with their own endpoints untouched.
fn rewrite_cf7_actions(html: &str, local: &str, path: &str) -> String {
    if !html.contains("wpcf7") {
        return html.to_string();
    }
    // Return path + slug for _page/tour hidden fields: `path` is the
    // page's rel dir ("booking/foo/" or "es/booking/foo/").
    let rel = format!("/{}", path.trim_end_matches('/'));
    let slug = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    let inject = format!(
        "<input type=\"hidden\" name=\"_page\" value=\"{rel}\" />\
         <input type=\"hidden\" name=\"tour\" value=\"{slug}\" />"
    );
    // Intentionally loose scan below: prefix-matches `<form-x`/
    // `<formdata` and is case-sensitive — CF7 origin output is always
    // lowercase `<form ` so a loose match is harmless in practice.
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(fi) = rest.find("<form") {
        out.push_str(&rest[..fi]);
        rest = &rest[fi..];
        let Some(end) = rest.find('>') else {
            out.push_str(rest);
            return out;
        };
        let tag = &rest[..end];
        let newtag = rewrite_tag_action(tag, local);
        let is_cf7 = tag.contains("wpcf7") || newtag != tag;
        out.push_str(&newtag);
        rest = &rest[end..];
        if is_cf7 {
            // rest begins with '>': emit it, then the hidden fields.
            out.push('>');
            out.push_str(&inject);
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

/// `action="…#wpcf7-…"` → `{local}/form/enquiry` inside one <form> tag.
/// Returns the tag unchanged when no CF7-fragment action is present.
fn rewrite_tag_action(tag: &str, local: &str) -> String {
    let Some(ai) = tag.find("action=") else {
        return tag.to_string();
    };
    let after = ai + 7;
    let bytes = tag.as_bytes();
    let (vstart, vend) = match bytes.get(after) {
        Some(b'"') | Some(b'\'') => {
            let q = bytes[after] as char;
            match tag[after + 1..].find(q) {
                Some(e) => (after + 1, after + 1 + e),
                None => return tag.to_string(),
            }
        }
        _ => {
            // Unquoted value: ends at whitespace or the tag's '>' —
            // `tag` excludes '>' so a missing-whitespace value is the
            // tag tail.
            match tag[after..].find(char::is_whitespace) {
                Some(e) => (after, after + e),
                None => (after, tag.len()),
            }
        }
    };
    if !tag[vstart..vend].contains("wpcf7") {
        return tag.to_string();
    }
    let mut t = tag.to_string();
    t.replace_range(vstart..vend, &format!("{local}/form/enquiry"));
    t
}
/// Fetch `origin + path`, extract the Revolution Slider block and
/// return it rewritten to `target` — or `Ok(None)` when the page
/// carries no `<rs-module-wrap>`. RevSlider stores slide data in
/// plugin tables outside the WP dump, so the rendered markup is the
/// only portable source; `rs6` JS/CSS are pulled with wp-content and
/// served from `www/`, so the fragment animates natively. Only the
/// FIRST slider is extracted (GoodLayers emits at most one
/// `revolution-slider` element per page in practice; a second one
/// would share the same harvest anyway).
pub(crate) fn revslider_fragment(origin: &str, path: &str, target: &str) -> Result<Option<String>> {
    let html = page(origin, path, target)?;
    let mut frag = extract_rs_module(&html);
    // Protocol-relative refs (//host/…) survive page()'s absolute-URL
    // rewrite — rs slides emit data-lazyload/src that way. Swap for the
    // target wholesale so `//origin/x` → `{{ site_url }}/x` renders as
    // an absolute local URL.
    if let (Some(f), Some(oh)) = (frag.as_mut(), origin_host(origin)) {
        *f = f.replace(&format!("//{oh}"), target);
    }
    Ok(frag)
}

fn origin_host(origin: &str) -> Option<&str> {
    origin
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .split('/')
        .next()
        .filter(|h| !h.is_empty())
}

/// `<rs-module-wrap>…</rs-module-wrap>` out of a full page, plus the
/// adjacent inline config blocks — origin emits, directly before the
/// wrap:
///   <style>…#rev_slider_N_M .uranus…</style>  (arrow/bullet skin)
///   <script>…window.RS_MODULES…</script>       (module config)
/// both of which the fragment needs to self-initialize. The wrap is
/// custom elements, so a plain close-tag search bounds it exactly.
fn extract_rs_module(html: &str) -> Option<String> {
    let Some(mut start) = html.find("<rs-module-wrap") else {
        return None;
    };
    let end_wrap = html.find("</rs-module-wrap>")? + "</rs-module-wrap>".len();
    // Walk back over immediately-preceding <style>/<script> blocks
    // (separated only by whitespace) that reference this slider's ids.
    loop {
        let back = &html[..start];
        let close = back.rfind("</style>").map(|i| (i, "</style>", "<style"))
            .into_iter()
            .chain(back.rfind("</script>").map(|i| (i, "</script>", "<script")))
            .max_by_key(|(i, _, _)| *i);
        let Some((ci, ctag, otag)) = close else { break };
        if !html[ci + ctag.len()..start].trim().is_empty() {
            break;
        }
        let back2 = &html[..ci];
        let Some(oi) = back2.rfind(otag) else { break };
        let block = &html[oi..ci + ctag.len()];
        // Only absorb blocks that clearly belong to this slider —
        // generic page scripts/styles stay with the page.
        if !(block.contains("rev_slider") || block.contains("RS_MODULES") || block.contains("uranus")) {
            break;
        }
        start = oi;
    }
    // The actual init lives in the footer as
    // <script id="rs-initialisation-scripts">…revapi2.revolutionInit…
    // — append it after the wrap so the fragment self-initializes
    // wherever the element lands.
    let mut frag = html[start..end_wrap].to_string();
    if let Some(si) = html.find("id=\"rs-initialisation-scripts\"") {
        if let Some(oi) = html[..si].rfind("<script") {
            if let Some(ce) = html[si..].find("</script>") {
                frag.push_str(&html[oi..si + ce + "</script>".len()]);
            }
        }
    }
    Some(frag)
}

/// Head assets the fragment needs — origin enqueues these globally;
/// our pages don't, so the element render prepends them. rs6 needs
/// `setREVStartSize` defined before the fragment's inline config
/// calls it (origin defines it in <head>); it's version-stable
/// boilerplate so we ship it verbatim rather than scrape it.
/// Plugin paths live under `www/wp-content/plugins/revslider/`.
pub(crate) const REVSLIDER_ASSETS: &str = concat!(
    "<link rel='stylesheet' href='{{ site_url }}/wp-content/plugins/revslider/public/assets/css/rs6.css' media='all' />",
    "<script src='{{ site_url }}/wp-content/plugins/revslider/public/assets/js/rbtools.min.js'></script>",
    "<script src='{{ site_url }}/wp-content/plugins/revslider/public/assets/js/rs6.min.js'></script>",
    "<script>",
    "function setREVStartSize(e){window.RSIW=window.RSIW===undefined?window.innerWidth:window.RSIW;window.RSIH=window.RSIH===undefined?window.innerHeight:window.RSIH;try{var pw=document.getElementById(e.c).parentNode.offsetWidth,newh;pw=pw===0||isNaN(pw)||(e.l==\"fullwidth\"||e.layout==\"fullwidth\")?window.RSIW:pw;e.tabw=e.tabw===undefined?0:parseInt(e.tabw);e.thumbw=e.thumbw===undefined?0:parseInt(e.thumbw);e.tabh=e.tabh===undefined?0:parseInt(e.tabh);e.thumbh=e.thumbh===undefined?0:parseInt(e.thumbh);e.tabhide=e.tabhide===undefined?0:parseInt(e.tabhide);e.thumbhide=e.thumbhide===undefined?0:parseInt(e.thumbhide);e.mh=e.mh===undefined||e.mh==\"\"||e.mh===\"auto\"?0:parseInt(e.mh,0);if(e.layout===\"fullscreen\"||e.l===\"fullscreen\")newh=Math.max(e.mh,window.RSIH);else{e.gw=Array.isArray(e.gw)?e.gw:[e.gw];for(var i in e.rl)if(e.gw[i]===undefined||e.gw[i]===0)e.gw[i]=e.gw[i-1];e.gh=e.el===undefined||e.el===\"\"||(Array.isArray(e.el)&&e.el.length==0)?e.gh:e.el;e.gh=Array.isArray(e.gh)?e.gh:[e.gh];for(var i in e.rl)if(e.gh[i]===undefined||e.gh[i]===0)e.gh[i]=e.gh[i-1];var nl=new Array(e.rl.length),ix=0,sl;e.tabw=e.tabhide>=pw?0:e.tabw;e.thumbw=e.thumbhide>=pw?0:e.thumbw;e.tabh=e.tabhide>=pw?0:e.tabh;e.thumbh=e.thumbhide>=pw?0:e.thumbh;for(var i in e.rl)nl[i]=e.rl[i]<window.RSIW?0:e.rl[i];sl=nl[0];for(var i in nl)if(sl>nl[i]&&nl[i]>0){sl=nl[i];ix=i;}var m=pw>(e.gw[ix]+e.tabw+e.thumbw)?1:(pw-(e.tabw+e.thumbw))/(e.gw[ix]);newh=(e.gh[ix]*m)+(e.tabh+e.thumbh);}var el=document.getElementById(e.c);if(el!==null&&el)el.style.height=newh+\"px\";el=document.getElementById(e.c+\"_wrapper\");if(el!==null&&el){el.style.height=newh+\"px\";el.style.display=\"block\";}}catch(e){console.log(\"Failure at Presize of Slider:\"+e)}};",
    "</script>",
);
