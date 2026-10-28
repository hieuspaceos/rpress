//! Audit-count verification for an rpress SQLite database.

#[allow(unused_imports)]
use crate::sql;
use anyhow::{bail, Result};
use rusqlite::Connection;
use std::path::Path;

fn check(c: &Connection, failed: &mut u32, name: &str, sql: &str, expect: i64) {
    let got: i64 = c.query_row(sql, [], |r| r.get(0)).unwrap_or(-1);
    let ok = if expect < 0 {
        "?"
    } else if got == expect {
        "OK"
    } else {
        *failed += 1;
        "MISMATCH"
    };
    println!("{:<44} got={:<8} expect={:<8} {}", name, got, expect, ok);
}

/// Tag-diff parity check: render each manifest page and compare its
/// structural tag sequence to the reference capture.
/// cdn-cgi email hashes are normalized (random per fetch).
pub fn verify_refs(db: &Path, templates: &Path, site_url: &str, tour_base: &str) -> Result<()> {
    let manifest: &[(&str, &str, Option<(&str, &str, &str)>)] = &[
        ("home", "home.html", Some(("homepage", "fr", "page"))),
        ("tour", "tour.html", Some(("x", "fr", "tour"))),
        ("post", "post.html", Some(("cai-be-dans-le-delta-du-mekong", "fr", "post"))),
        ("page-form", "page-form.html", Some(("rappel-gratuit", "fr", "page"))),
        ("reviews", "reviews.html", Some(("avis-des-clients", "fr", "page"))),
        ("archive", "archive.html", Some(("profil/voyage-en-famille", "fr", "archive"))),
        ("home-es", "home-es.html", Some(("es", "es", "page"))),
    ];
    let refdir = templates.join("reference");
    println!("{:<10} {:>7} {:>8} {:>8}", "page", "match%", "ref", "out");
    let mut all_ok = true;
    for (label, file, spec) in manifest {
        let refp = refdir.join(file);
        let refh = match std::fs::read_to_string(&refp) {
            Ok(h) => h,
            Err(_) => {
                println!("{label:<10} no reference {file}");
                continue;
            }
        };
        let Some((slug, lang, kind)) = spec else {
            println!("{label:<10} unrenderable");
            all_ok = false;
            continue;
        };
        let ours = match crate::render::render_slug(db, templates, slug, lang, kind, site_url, tour_base) {
            Ok((h, _, _)) => h,
            Err(e) => {
                println!("{label:<10} render error: {e}");
                all_ok = false;
                continue;
            }
        };
        let r = tags(&normalize(&refh));
        let o = tags(&normalize(&ours));
        let (pct, top) = diff_ratio(&r, &o);
        if pct < 100.0 {
            all_ok = false;
        }
        println!("{label:<10} {pct:>6.1}% {:>8} {:>8}", r.len(), o.len());
        for (kind, a, b, ctx) in top {
            println!("    {kind} ref[{a}] out[{b}]  …{ctx}…");
        }
    }
    if !all_ok {
        bail!("verify-refs: parity below 100%");
    }
    Ok(())
}

/// Strip things that can never byte-match: CF email-protection hashes.
fn normalize(h: &str) -> String {
    const PAT: &str = "cdn-cgi/l/email-protection#";
    let mut out = String::with_capacity(h.len());
    let mut rest = h;
    while let Some(p) = rest.find(PAT) {
        out.push_str(&rest[..p + PAT.len()]);
        let tail = &rest[p + PAT.len()..];
        let hex_len = tail
            .bytes()
            .take_while(|b| b.is_ascii_hexdigit() || *b == b'-')
            .count();
        rest = &tail[hex_len..];
    }
    out.push_str(rest);
    out
}

/// Structural tag sequence (same metric as the parity measurements).
fn tags(h: &str) -> Vec<String> {
    let mut out = Vec::new();
    let b = h.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' && i + 1 < b.len() && (b[i + 1].is_ascii_alphabetic() || b[i + 1] == b'/') {
            let mut j = i + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-' || b[j] == b'/') {
                j += 1;
            }
            let name = &h[i..j];
            let n = name.trim_start_matches("</").trim_start_matches('<');
            if matches!(
                n,
                "div" | "section" | "ul" | "li" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "a"
                    | "span" | "img" | "p" | "i" | "iframe" | "table" | "tr" | "td" | "form"
                    | "input" | "select" | "option" | "button"
            ) {
                out.push(name.trim_start_matches('<').to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// LCS ratio + top mismatch windows.
fn diff_ratio(r: &[String], o: &[String]) -> (f64, Vec<(String, usize, usize, String)>) {
    let ops = lcs_ops(r, o);
    let equal: usize = ops.iter().filter(|(k, _, _, _, _)| *k == ' ').map(|(_, a, b, _, _)| b - a).sum();
    let pct = if r.is_empty() { 100.0 } else { 100.0 * equal as f64 / r.len() as f64 };
    let mut top = Vec::new();
    for (k, a1, a2, b1, b2) in ops.iter().filter(|(k, ..)| *k != ' ').take(10) {
        let ctx = if *a1 < r.len() { r[*a1].clone() } else { "∅".into() };
        top.push((k.to_string(), *a1, *b1, format!("{ctx} (+{} ref, +{} out)", a2 - a1, b2 - b1)));
    }
    (pct, top)
}

/// O(n·m) LCS opcode walk — fine at ~2k tags.
fn lcs_ops(a: &[String], b: &[String]) -> Vec<(char, usize, usize, usize, usize)> {
    let (n, m) = (a.len(), b.len());
    let mut dp = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] { dp[i + 1][j + 1] + 1 } else { dp[i + 1][j].max(dp[i][j + 1]) };
        }
    }
    let mut ops = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            let s = i;
            while i < n && j < m && a[i] == b[j] {
                i += 1;
                j += 1;
            }
            ops.push((' ', s, i, s, j));
        } else {
            let (si, sj) = (i, j);
            while i < n && j < m && a[i] != b[j] {
                if dp[i + 1][j] >= dp[i][j + 1] { i += 1; } else { j += 1; }
            }
            if i >= n || j >= m {
                while i < n { i += 1; }
                while j < m { j += 1; }
            }
            ops.push(('≠', si, i, sj, j));
        }
    }
    if i < n || j < m {
        ops.push(('≠', i, n, j, m));
    }
    ops
}

pub fn run(db: &Path) -> Result<()> {
    let c = crate::db::conn(db)?;
    let mut failed = 0u32;

    println!("== content counts ==");
    // Expected counts are recorded per-site at parse time in
    // settings.import_expect — never constants, so any dump verifies.
    let expect_json: Option<String> = c
        .query_row(sql!("render__select_value_from_settings"), rusqlite::params!["import_expect"], |r| r.get(0))
        .ok();
    match expect_json.and_then(|j| serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&j).ok()) {
        None => println!("(no import_expect — content counts not checked; re-import to record them)"),
        Some(expect) => {
            let mut keys: Vec<&String> = expect.keys().collect();
            keys.sort();
            for k in keys {
                let n = expect[k].as_i64().unwrap_or(0);
                if let Some(rest) = k.strip_prefix("posts:") {
                    let (kind, lang) = rest.rsplit_once(':').unwrap_or((rest, ""));
                    check(&c, &mut failed, &format!("{kind} {lang}"),
                        &format!("SELECT COUNT(*) FROM posts WHERE kind='{kind}' AND lang='{lang}' AND status IN ('publish','draft')"), n);
                } else {
                    let (label, sql) = match k.as_str() {
                        "comments_total" => ("comments", "SELECT COUNT(*) FROM comments"),
                        "menu_items" => ("menu_items", "SELECT COUNT(*) FROM menu_items"),
                        "menus" => ("menus", "SELECT COUNT(*) FROM menus"),
                        "media" => ("media", "SELECT COUNT(*) FROM media"),
                        "translation_members" => ("translation members", "SELECT COUNT(*) FROM post_translation_members"),
                        _ => continue,
                    };
                    check(&c, &mut failed, label, sql, n);
                }
            }
            /* Widgets compare with `>=` like redirects: admins can add
               widget rows post-import without breaking parity. */
            let want_w: i64 = expect.get("widgets").and_then(|v| v.as_i64()).unwrap_or(0);
            let widgets: i64 = c.query_row("SELECT COUNT(*) FROM widgets", [], |r| r.get(0)).unwrap_or(0);
            let widgets_ok = widgets >= want_w;
            println!("{:<44} got={:<8} expect={:<8} {}", format!("widgets >= {want_w}"), widgets, want_w, if widgets_ok { "OK" } else { "MISMATCH" });
            if !widgets_ok {
                failed += 1;
            }
            let want: i64 = expect.get("redirects").and_then(|v| v.as_i64()).unwrap_or(0);
            let redir: i64 = c.query_row(sql!("verify__select_count_from_1"), [], |r| r.get(0)).unwrap_or(0);
            let redir_ok = redir >= want;
            println!("{:<44} got={:<8} expect={:<8} {}", format!("redirects >= {want}"), redir, want, if redir_ok { "OK" } else { "MISMATCH" });
            if !redir_ok {
                failed += 1;
            }
        }
    }

    println!("\n== integrity ==");
    // wp_id IS NULL on runtime-created rows (last_insert_rowid convention,
    // M12) — the id==wp_id invariant only applies to imported rows.
    check(&c, &mut failed, "id == wp_id (posts)", "SELECT COUNT(*) FROM posts WHERE wp_id IS NOT NULL AND id != wp_id", 0);
    check(&c, &mut failed, "id == wp_id (comments)", "SELECT COUNT(*) FROM comments WHERE wp_id IS NOT NULL AND id != wp_id", 0);
    check(&c, &mut failed, "id == wp_id (media)", "SELECT COUNT(*) FROM media WHERE wp_id IS NOT NULL AND id != wp_id", 0);
    check(&c, &mut failed, "comments join posts.id",
        "SELECT COUNT(*) FROM comments cm WHERE NOT EXISTS (SELECT 1 FROM posts p WHERE p.id=cm.post_id)", 0);
    check(&c, &mut failed, "menu parent join items.id",
        "SELECT COUNT(*) FROM menu_items m WHERE m.parent_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM menu_items p WHERE p.id=m.parent_id)", 0);
    check(&c, &mut failed, "menu_id join menus.id",
        "SELECT COUNT(*) FROM menu_items m WHERE NOT EXISTS (SELECT 1 FROM menus n WHERE n.id=m.menu_id)", 0);
    check(&c, &mut failed, "translation members resolve",
        "SELECT COUNT(*) FROM post_translations t WHERE (t.fr_post_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM posts p WHERE p.id=t.fr_post_id)) OR (t.es_post_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM posts p WHERE p.id=t.es_post_id))", 0);
    check(&c, &mut failed, "body_json invalid",
        "SELECT COUNT(*) FROM posts WHERE json_valid(body_json)=0", 0);
    // posts whose lang isn't in the imported languages set (N-lang generic)
    check(&c, &mut failed, "posts without lang",
        "SELECT COUNT(*) FROM posts p WHERE NOT EXISTS (
           SELECT 1 FROM settings s WHERE s.key='languages'
             AND (','||s.value||',') LIKE ('%,'||p.lang||',%'))", 0);
    // page_on_front only matters when the front page is a static page —
    // show_on_front='posts' (WP's default) legitimately leaves it unset/0.
    let show_on_front: String = c
        .query_row(sql!("render__select_value_from_settings"), rusqlite::params!["show_on_front"], |r| r.get(0))
        .unwrap_or_else(|_| "posts".into());
    if show_on_front == "page" {
        check(&c, &mut failed, "settings.page_on_front set",
            "SELECT COUNT(*) FROM settings WHERE key='page_on_front' AND value != '0'", 1);
    } else {
        println!("{:<44} {}", "settings.page_on_front set", "skipped (show_on_front=posts)");
    }
    let seo: i64 = c.query_row(
        sql!("verify__select_cast_count"),
        [], |r| r.get(0)).unwrap_or(-1);
    println!("{:<44} got={:<8} expect={:<8} {}", "seo_title coverage %", seo, -1, "?");

    let front: i64 = c.query_row(
        sql!("verify__select_cast_value_3"),
        [], |r| r.get(0)).unwrap_or(0);
    let front_kind: String = c.query_row(
        sql!("verify__select_kind_from"), [front], |r| r.get(0)).unwrap_or_default();
    println!("page_on_front → id {} kind={}", front, front_kind);

    println!("\n== db size ==");
    let sz = std::fs::metadata(db).map(|m| m.len()).unwrap_or(0);
    println!("database = {:.1} MB (limit 500)", sz as f64 / 1e6);
    if sz as f64 > 500e6 {
        failed += 1;
    }

    if failed > 0 {
        bail!("{failed} verify check(s) failed");
    }
    Ok(())
}
