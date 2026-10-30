#[allow(unused_imports)]
use crate::sql;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};



#[derive(Clone, Default)]
pub struct Rate {
    hits: HashMap<String, Vec<Instant>>,
}

/// Hard cap on tracked IPs so a spray of spoofed peers can't grow the map
/// unboundedly. On full, evict the *stalest* key instead of denying new IPs:
/// deny-new let a >10k-IP spray lock the whole internet out (self-inflicted
/// DoS). For per-IP counters the evict-stalest semantics are safe — see
/// `login_key` for the login path where eviction alone is NOT enough.
const RATE_MAX_KEYS: usize = 10_000;

/// Rate-limit key for *login attempts*: `/24` (IPv4) or `/64` (IPv6) subnet,
/// not the bare IP. Evict-stalest without this lets a ≥10k-IP botnet evict
/// its own entries and reset its own counter to zero — turning the 20/min
/// cap into effectively uncapped brute force. Subnet keying makes hosts in
/// one prefix share one budget.
pub fn login_key(ip: &str) -> String {
    if let Some(v4) = ip.parse::<std::net::Ipv4Addr>().ok() {
        let o = v4.octets();
        return format!("{}.{}.{}.0/24", o[0], o[1], o[2]);
    }
    if let Some(v6) = ip.parse::<std::net::Ipv6Addr>().ok() {
        let s = v6.segments();
        return format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3]);
    }
    ip.to_string()
}

impl Rate {
    pub fn allow(&mut self, ip: &str, max: usize, win: Duration) -> bool {
        let now = Instant::now();
        if !self.hits.contains_key(ip) && self.hits.len() >= RATE_MAX_KEYS {
            self.hits.retain(|_, ts| ts.iter().any(|t| now.duration_since(*t) < win));
            if self.hits.len() >= RATE_MAX_KEYS {
                // Evict the stalest key — bounded LRU, never deny unseen IPs.
                if let Some((old, _)) = self
                    .hits
                    .iter()
                    .min_by_key(|(_, ts)| ts.iter().max().copied())
                    .map(|(k, ts)| (k.clone(), ts.clone()))
                {
                    self.hits.remove(&old);
                }
            }
        }
        let e = self.hits.entry(ip.to_string()).or_default();
        e.retain(|t| now.duration_since(*t) < win);
        if e.len() >= max {
            return false;
        }
        e.push(now);
        true
    }
}

/// `allowed` is built per site from `settings.site_url` (+ its `edit.` sibling).
/// Localhost always allowed for dev.
/// Full-origin equality only (A7): `starts_with` let `https://site.com.evil.com`
/// pass; trailing slashes are normalized before comparing.
pub fn cors_ok(allowed: &[String], origin: Option<&str>) -> bool {
    match origin {
        Some(o) => {
            let o = o.trim_end_matches('/');
            allowed
                .iter()
                .any(|p| o == p.trim_end_matches('/'))
                || o.starts_with("http://127.0.0.1:")
                || o.starts_with("http://localhost:")
        }
        None => true, // curl / same-origin
    }
}

fn insert(
    db: &Path,
    kind: &str,
    name: &str,
    page: &str,
    tour: &str,
    data: &str,
    ip: &str,
    form_slug: Option<&str>,
) -> Result<i64> {
    let c = crate::db::conn(db)?;
    c.execute(
        sql!("intake__insert_into_submissions"),
        params![kind, name, page, tour, data, ip, form_slug],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn submit_form(db: &Path, slug: &str, fields: &HashMap<String, String>, ip: &str) -> Result<&'static str> {
    let form = crate::db::forms::get_by_slug(db, slug)?
        .ok_or_else(|| anyhow::anyhow!("unknown form"))?;
    if fields.get("_gotcha").map(|s| !s.is_empty()).unwrap_or(false) {
        return Ok("ok");
    }
    // Validate required + type rules from the markup tags.
    let tags = crate::forms_markup::parse(&form.markup)
        .map_err(|e| anyhow::anyhow!("bad markup: {e}"))?;
    for t in &tags {
        // Captcha markers + hidden fields don't validate.
        if matches!(t.type_.as_str(), "submit" | "cf7sr-turnstile" | "turnstile" | "hidden" | "dynamictext") {
            continue;
        }
        // Quiz: CF7 `[quiz name "question|answer"]` — accept when the
        // posted value matches the answer (case-insensitive); the
        // tag's last quoted opt carries `question|answer`.
        if t.type_ == "quiz" {
            let v = fields.get(&t.name).cloned().unwrap_or_default();
            // CF7 `[quiz]` may carry several "q|a" pairs and picks one
            // at random; the renderer always shows the FIRST pair
            // (forms_markup::tag_html), so validate against that same
            // pair — text after '|' inside the first quoted segment.
            let joined = t.opts.join(" ");
            let first_quoted = joined.split('"').nth(1).unwrap_or("");
            let want = first_quoted.rsplit('|').next().unwrap_or("").trim().to_string();
            if !want.is_empty() && !v.trim().eq_ignore_ascii_case(&want) {
                anyhow::bail!("quiz_failed:{}", t.name);
            }
            continue;
        }
        // checkboxes/radios post as `name[]`; everything else as `name`.
        let v = fields.get(&t.name)
            .or_else(|| fields.get(&format!("{}[]", t.name)))
            .cloned().unwrap_or_default();
        if t.required && v.is_empty() {
            anyhow::bail!("required:{}", t.name);
        }
        // phonetext validates like tel; countrytext is free text.
        let vtype = if t.type_ == "phonetext" { "tel" } else { t.type_.as_str() };
        if let Some(key) = crate::forms_markup::check_value(vtype, &v) {
            anyhow::bail!("{key}:{}", t.name);
        }
    }
    // CF7 name fields: `your-name`, `your-email`, `your-message`, `your-subject`.
    // Map them to our stored columns (name / email in `data_json`).
    let name = fields
        .get("your-name")
        .or_else(|| fields.get("name"))
        .cloned()
        .unwrap_or_default();
    let page = fields.get("_page").cloned().unwrap_or_default();
    let tour = fields
        .get("tour-name")
        .or_else(|| fields.get("tour"))
        .cloned()
        .unwrap_or_default();
    let data = serde_json::to_string(fields)?;
    insert(db, &form.kind, &name, &page, &tour, &data, ip, Some(slug))?;
    Ok("ok")
}

/// Outcome of a public comment POST — feeds both the redirect anchor and the
/// optional post-page regen the handler schedules.
pub enum CommentResult {
    /// Stored + approved: the rendered page is stale the moment we return
    /// → the caller regenerates the post file.
    Submitted,
    /// Stored pending (manual moderation, moderation keys, max links, or a
    /// first-time author under `comment_previously_approved`) — or quietly
    /// trashed by `disallowed_keys` (the user still gets the held message).
    Held,
    /// Rejected politely: comments closed by age or reply depth exceeded.
    /// Not an error — the page redirects to the closed note.
    Closed,
}

/// `settings` row read with a default (same store `admin/settings.rs`
/// writes — discussion options live there).
pub(crate) fn setting(c: &Connection, key: &str, default: &str) -> String {
    c.query_row(sql!("intake__select_setting"), [key], |r| r.get(0))
        .unwrap_or_else(|_| default.to_string())
}

/// Public comment intake (`POST /comment`). Discussion settings are
/// enforced here — the same order WP runs them in `wp_new_comment` /
/// `wp_allow_comment`: disallowed → trash, moderation keys / max links →
/// pending, previously-approved whitelist → approved, manual moderation →
/// pending, else approved.
///
/// Hidden-field contract (templates/comments_wpdiscuz.html):
///   post_id  — target post id (primary; `_page` URL is the fallback)
///   _page    — post permalink, also the redirect target
///   parent   — 0 or an approved comment id on the same post
///   wc_name/wc_email/wc_comment (+ plain name/email/comment aliases)
///   url      — optional author URL; non-http(s) schemes render unlinked
///   _gotcha  — honeypot, filled = silently "accepted"
pub fn submit_comment(db: &Path, fields: &HashMap<String, String>, ip: &str) -> Result<(CommentResult, i64)> {
    if fields.get("_gotcha").map(|s| !s.is_empty()).unwrap_or(false) {
        return Ok((CommentResult::Submitted, 0));
    }
    let field = |a: &str, b: &str| {
        fields.get(a).or_else(|| fields.get(b)).cloned().unwrap_or_default()
    };
    let name = field("wc_name", "name");
    let email = field("wc_email", "email");
    let body = field("wc_comment", "comment");
    let url = field("url", "author_url");
    if name.is_empty() || body.is_empty() {
        anyhow::bail!("missing fields");
    }
    let c = crate::db::conn(db)?;

    // Target post: `post_id` hidden field first (exact), `_page` URL second.
    let (post_id, published): (i64, String) =
        match fields.get("post_id").and_then(|v| v.parse::<i64>().ok()) {
            Some(id) if id > 0 => c
                .query_row(sql!("intake__select_post_by_id"), [id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .map_err(|_| anyhow::anyhow!("unknown page"))?,
            _ => {
                let pid = resolve_page(&c, &field("_page", "page"))?;
                let p = c
                    .query_row(sql!("intake__select_post_published"), [pid], |r| {
                        r.get(0)
                    })
                    .unwrap_or_default();
                (pid, p)
            }
        };

    // --- discussion settings -------------------------------------------------
    let dcs = setting(&c, "default_comment_status", "open");
    // WP stores 'open'; imported rows can carry '1' — only an explicit
    // closed value locks the form.
    if dcs == "closed" || dcs == "0" {
        return Ok((CommentResult::Closed, post_id));
    }
    if setting(&c, "require_name_email", "1") == "1" && email.is_empty() {
        anyhow::bail!("missing fields");
    }
    let close_days: i64 = setting(&c, "close_comments_days_old", "14")
        .parse()
        .unwrap_or(14);
    if setting(&c, "close_comments", "0") == "1" && close_days > 0 && days_old(&published) > close_days {
        return Ok((CommentResult::Closed, post_id));
    }

    // Parent validity + thread depth (WP rejects replies past
    // `thread_comments_depth`; `thread_comments=0` means no nesting at all).
    let parent: i64 = field("parent", "comment_parent").parse().unwrap_or(0);
    if parent != 0 {
        let mut depth = 0i64; // ancestors of the new comment
        let mut cur = parent;
        let mut valid = true;
        while cur != 0 {
            depth += 1;
            if depth > 64 {
                valid = false;
                break;
            }
            cur = match c
                .query_row(
                    sql!("intake__select_comment_parent"),
                    params![cur, post_id],
                    |r| r.get::<_, Option<i64>>(0),
                )
                .optional()?
            {
                Some(p) => p.unwrap_or(0),
                None => {
                    valid = false;
                    break;
                }
            };
        }
        if !valid {
            anyhow::bail!("bad parent");
        }
        let max = if setting(&c, "thread_comments", "1") == "1" {
            setting(&c, "thread_comments_depth", "5")
                .parse::<i64>()
                .unwrap_or(5)
                .max(1)
        } else {
            1
        };
        if depth + 1 > max {
            return Ok((CommentResult::Closed, post_id));
        }
    }

    // Moderation wordlists — substring match on every submitted field
    // (WP checks content, name, url, email and IP the same way).
    let hay = format!("{name}\n{email}\n{url}\n{body}\n{ip}").to_lowercase();
    let hits = |keys: &str| -> bool {
        keys.lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .any(|k| hay.contains(&k))
    };
    let max_links: usize = setting(&c, "comment_max_links", "2")
        .parse()
        .unwrap_or(2);
    let link_count = body.matches("http").count();

    let status = if hits(&setting(&c, "disallowed_keys", "")) {
        "trash"
    } else if hits(&setting(&c, "moderation_keys", "")) || link_count > max_links {
        "pending"
    } else if setting(&c, "comment_previously_approved", "0") == "1"
        && !email.is_empty()
        && c.query_row(
            sql!("intake__count_approved_by_email"),
            [&email.to_lowercase()],
            |r| r.get::<_, i64>(0),
        )? > 0
    {
        "approved"
    } else if setting(&c, "comment_moderation", "0") == "1" {
        "pending"
    } else {
        "approved"
    };

    // Runtime comment intake (concurrent writer): SQLite assigns the id —
    // the offline-import `MAX(id)+1` pattern races here (audit M12). The
    // new comment's id is never read back, so no last_insert_rowid() needed.
    c.execute(
        sql!("intake__insert_comment"),
        params![
            post_id,
            if parent == 0 { None } else { Some(parent) },
            name,
            email,
            body,
            url,
            ip,
            status,
        ],
    )?;
    match status {
        "approved" => Ok((CommentResult::Submitted, post_id)),
        // Trashed-by-wordlist reads as held to the submitter (WP does the
        // same — disallowed comments get the moderation message).
        _ => Ok((CommentResult::Held, post_id)),
    }
}

/// Whole days between a stored `YYYY-MM-DD HH:MM:SS` (UTC) and now, for
/// `close_comments_days_old`. Civil-calendar math (Howard Hinnant's
/// days_from_civil); unparsable dates age to "ancient" so a close check
/// rejects them rather than staying open.
pub(crate) fn days_old(published: &str) -> i64 {
    let y: i64 = published.get(0..4).and_then(|s| s.parse().ok()).unwrap_or(0);
    if y == 0 {
        return i64::MAX / 4;
    }
    let m: i64 = published.get(5..7).and_then(|s| s.parse().ok()).unwrap_or(1);
    let d: i64 = published.get(8..10).and_then(|s| s.parse().ok()).unwrap_or(1);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let z = era * 146097 + doe - 719468;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    now / 86400 - z
}

pub fn submit_review(db: &Path, fields: &HashMap<String, String>, ip: &str) -> Result<&'static str> {
    if fields.get("_gotcha").map(|s| !s.is_empty()).unwrap_or(false) {
        return Ok("ok");
    }
    let rating: i64 = fields.get("rating").and_then(|s| s.parse().ok()).unwrap_or(0);
    if !(1..=5).contains(&rating) {
        anyhow::bail!("rating 1-5");
    }
    let name = fields.get("name").cloned().unwrap_or_default();
    let body = fields.get("review").cloned().unwrap_or_default();
    if name.is_empty() || body.is_empty() {
        anyhow::bail!("missing fields");
    }
    let page = fields.get("_page").cloned().unwrap_or_default();
    let data = serde_json::to_string(fields)?;
    insert(db, "review", &name, &page, "", &data, ip, None)?;
    Ok("ok")
}

#[derive(Deserialize)]
pub struct Moderation {
    pub id: i64,
    pub action: String,
    pub csrf: String,
}

/// Returns the post id a caller should regenerate (approving a comment or
/// review submission changes a rendered page); 0 when nothing on the site
/// changed.
pub fn moderate(db: &Path, id: i64, action: &str) -> Result<i64> {
    let c = crate::db::conn(db)?;
    let mut dirty_post: i64 = 0;
    match action {
        // Legacy "delete" accepted: it was the old trash word.
        "spam" | "trash" | "delete" => {
            let status = if action == "spam" { "spam" } else { "trash" };
            c.execute(
                sql!("intake__update_submissions_set"),
                params![status, id],
            )?;
        }
        "restore" => {
            c.execute(sql!("intake__update_submissions_set"), params!["new", id])?;
        }
        "purge" => {
            c.execute(sql!("intake__delete_from_submissions"), [id])?;
        }
        "approve" => {
            let (kind, name, page, data, ip): (String, String, String, String, String) = c.query_row(
                sql!("intake__select_submission_approve"),
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?;
            let map: HashMap<String, String> = serde_json::from_str(&data).unwrap_or_default();
            match kind.as_str() {
                "comment" => {
                    let post_id = resolve_page(&c, &page)?;
                    dirty_post = post_id;
                    let parent: i64 = map.get("parent").and_then(|s| s.parse().ok()).unwrap_or(0);
                    // SQLite assigns the comment id — MAX(id)+1 raced under
                    // concurrent writers (audit M12).
                    c.execute(
                        sql!("intake__insert_into_comments"),
                        params![
                            post_id,
                            if parent == 0 { None } else { Some(parent) },
                            name,
                            map.get("email").cloned().unwrap_or_default(),
                            map.get("comment").cloned().unwrap_or_default(),
                            map.get("url").cloned().unwrap_or_default(),
                            ip,
                        ],
                    )?;
                }
                "review" => {
                    let post_id = resolve_page(&c, &page).unwrap_or(0);
                    dirty_post = post_id;
                    let body = serde_json::json!([{
                        "template":"element",
                        "type":"text-box",
                        "value":{"content": map.get("review").cloned().unwrap_or_default()}
                    }])
                    .to_string();
                    // SQLite assigns the post id — MAX(id)+1 raced under
                    // concurrent writers (audit M12); slug stamped after.
                    // tx: a failed slug UPDATE must not leave a review post
                    // parked on the shared placeholder slug.
                    {
                        let tx = c.unchecked_transaction()?;
                        tx.execute(
                            sql!("intake__insert_into_posts_6"),
                            params![name, body],
                        )?;
                        let next = tx.last_insert_rowid();
                        tx.execute(
                            "UPDATE posts SET slug=?1 WHERE id=?2",
                            params![format!("review-{next}"), next],
                        )?;
                        if post_id > 0 {
                            tx.execute(
                                sql!("intake__insert_or_ignore_7"),
                                params![next, post_id],
                            )?;
                        }
                        tx.commit()?;
                    }
                }
                _ => {}
            }
            c.execute(sql!("intake__update_submissions_set_8"), [id])?;
        }
        _ => anyhow::bail!("bad action"),
    }
    Ok(dirty_post)
}

fn resolve_page(c: &Connection, page: &str) -> Result<i64> {
    // Page URL carries its language prefix (/es/...): FR+ES posts can share
    // a slug, so scope the lookup or a Spanish comment would land on the FR
    // twin. No prefix → fr.
    let clean = page.trim().trim_matches('/');
    let cfg = crate::lang::config(c);
    let (lang, rest) = crate::lang::split_path(&cfg, clean);
    let slug = rest.rsplit('/').next().unwrap_or(rest);
    let scoped = c.query_row(
        sql!("intake__select_id_from_9"),
        rusqlite::params![slug, lang],
        |r| r.get::<_, i64>(0),
    );
    scoped.or_else(|_| {
        // forms that post a bare slug (no /es prefix) still resolve — fall
        // back to the unscoped lookup only when the scoped one missed.
        c.query_row("SELECT id FROM posts WHERE slug=?1 AND status='publish' LIMIT 1", [slug], |r| {
            r.get(0)
        })
    })
    .map_err(|_| anyhow::anyhow!("unknown page"))
}
