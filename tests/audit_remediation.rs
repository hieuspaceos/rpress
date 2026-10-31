//! Shared regression home for the 2026-10-31 audit remediation
//! (`plans/261031-0450-audit-1031-remediation/`). Each phase lands its
//! proving test here unless an existing test file owns the subject.

#![allow(dead_code)] // scaffold: later phases consume these helpers

use std::path::{Path, PathBuf};

/// Per-test temp dir under `rpress-audit-<pid>-<name>` — same convention as
/// `tests/import_expect.rs::dir`.
fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rpress-audit-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Fresh fully-migrated database (DDL + column migrations + STEPS).
fn migrated_db(name: &str) -> (PathBuf, rusqlite::Connection) {
    let d = dir(name);
    let db = d.join("rpress.db");
    let c = rusqlite::Connection::open(&db).unwrap();
    rpress::schema::migrate_conn(&c).unwrap();
    (db, c)
}

/// Insert one session row; `expires_at` is a unix epoch (0 = never expires,
fn insert_session(c: &rusqlite::Connection, token: &str, expires_at: i64) {
    c.execute(
        "INSERT INTO sessions(token,user_id,ua,ip,csrf,created_at,last_seen,expires_at)
         VALUES(?1,1,'ua','127.0.0.1','',datetime('now'),datetime('now'),?2)",
        rusqlite::params![token, expires_at],
    )
    .unwrap();
}

fn session_count(c: &rusqlite::Connection) -> i64 {
    c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap()
}

/// Write `bytes` to `<dir>/dump.sql` and return the path.
fn write_dump(d: &Path, bytes: &[u8]) -> PathBuf {
    let p = d.join("dump.sql");
    std::fs::write(&p, bytes).unwrap();
    p
}

// ---- phase 06: tourmaster panics/wipe, spec metabox footguns ----

/// `date-price` month values are filtered to 1..=12 at parse: 13/0 are
/// skipped (no weekday_of OOB), valid months still expand.
#[test]
fn phase06_month_filtered_at_parse() {
    let entry = serde_json::json!({
        "day": ["monday"],
        "month": ["13", "0", "3"],
        "year": ["2026"],
    });
    let mut out = Vec::new();
    rpress::plugins::tourmaster::save::expand_entry(
        entry.as_object().unwrap(),
        "repeat",
        &mut out,
    );
    assert!(!out.is_empty(), "valid month 3 produced no dates");
    assert!(
        out.iter().all(|d| d.starts_with("2026-03-")),
        "out-of-range months leaked: {out:?}"
    );
    // Sanity on the guard target itself.
    assert_eq!(rpress::plugins::tourmaster::save::weekday_of(2026, 3, 2), 1);
}

/// search_months emits strictly increasing YYYY-MM slugs and rolls the
/// year at most once per slot — no compounding year bug.
#[test]
fn phase06_search_months_rollover_sanity() {
    let months = rpress::plugins::tourmaster::render::search_months(
        14,
        rpress::lang::month_names("fr"),
    );
    assert_eq!(months.len(), 14);
    let slugs: Vec<String> = months
        .iter()
        .map(|m| m["slug"].as_str().unwrap().to_string())
        .collect();
    let mut prev: Option<(i64, i64)> = None;
    for s in &slugs {
        let (y, m) = s.split_once('-').unwrap();
        let (y, m) = (y.parse::<i64>().unwrap(), m.parse::<i64>().unwrap());
        assert!((1..=12).contains(&m), "bad month slug {s}");
        if let Some((py, pm)) = prev {
            let next = py * 12 + pm; // prev total + 1
            assert_eq!(y * 12 + (m - 1), next, "non-consecutive at {prev:?} -> {s}");
        }
        prev = Some((y, m));
    }
}

/// A stored multi-byte color has byte-len 6 but must never panic on the
/// char-boundary slice — empty string, no 500.
#[test]
fn phase06_hex_rgba_non_ascii() {
    assert_eq!(
        rpress::plugins::tourmaster::render::hex_rgba("#12é456", 0.5),
        ""
    );
    assert_eq!(
        rpress::plugins::tourmaster::render::hex_rgba("#10873b", 0.5),
        "rgba(16, 135, 59, 0.5)"
    );
}

/// A metabox posting only the `__present` marker (all checkboxes
/// unchecked) still zeroes its checkbox keys — before the marker a
/// checkbox-only box was un-uncheckable.
#[test]
fn phase06_spec_metabox_present_uncheck() {
    let (db, c) = migrated_db("phase06-spec");
    c.execute_batch(
        "INSERT INTO settings(key,value) VALUES
         ('languages','fr'),('primary_lang','fr'),
         ('site_url','http://127.0.0.1'),('site_name','AuditTest');",
    )
    .unwrap();
    // Seed a paid booking, then uncheck via a metabox-shaped save.
    let f: rpress::db::posts::PostForm = serde_urlencoded::from_str(
        "kind=to_book&title=B&slug=b&lang=fr&status=publish&__present=1&ba_booking_paid=1",
    )
    .unwrap();
    let id = rpress::db::posts::save_post(&db, &f).unwrap();
    let meta = |k: &str| -> Option<String> {
        c.query_row(
            "SELECT meta_value FROM post_meta WHERE post_id=?1 AND meta_key=?2",
            rusqlite::params![id, k],
            |r| r.get(0),
        )
        .ok()
    };
    assert_eq!(meta("ba_booking_paid").as_deref(), Some("1"));

    let f2: rpress::db::posts::PostForm = serde_urlencoded::from_str(&format!(
        "kind=to_book&title=B&slug=b&lang=fr&status=publish&id={id}&__present=1"
    ))
    .unwrap();
    rpress::db::posts::save_post(&db, &f2).unwrap();
    assert_eq!(
        meta("ba_booking_paid").as_deref(),
        Some("0"),
        "checkbox-only metabox could not uncheck"
    );
}

// --- Phase 2: security hotfixes ------------------------------------------

/// `_page=/\evil.com` must not become the post-POST redirect target —
/// some stacks normalize `/\host` to `//host`, a protocol-relative bounce.
/// Query/fragment paths stay allowed (legit `_page` values carry them).
#[test]
fn safe_back_rejects_backslash_and_offsite() {
    assert_eq!(rpress::admin::safe_back(Some("/\\evil.com")), "/");
    assert_eq!(rpress::admin::safe_back(Some("//evil.com")), "/");
    assert_eq!(rpress::admin::safe_back(Some("https://evil.com/x")), "/");
    assert_eq!(rpress::admin::safe_back(Some("evil.com")), "/");
    assert_eq!(rpress::admin::safe_back(None), "/");
    assert_eq!(rpress::admin::safe_back(Some("/post?x=1#y")), "/post?x=1#y");
}

/// `Host:` normalization — `host:port` strips the port; `[v6]` and
/// `[v6]:port` yield the bare v6 literal; a bare v6 host keeps colons
/// (its suffix isn't a u16 port).
#[test]
fn request_host_normalizes_ipv6_and_port() {
    use axum::http::HeaderMap;
    let host = |v: &str| {
        let mut h = HeaderMap::new();
        h.insert(axum::http::header::HOST, v.parse().unwrap());
        rpress::admin::request_host(&h)
    };
    assert_eq!(host("Example.COM").as_deref(), Some("example.com"));
    assert_eq!(host("example.com:8443").as_deref(), Some("example.com"));
    assert_eq!(host("[::1]:8080").as_deref(), Some("::1"));
    assert_eq!(host("[2001:DB8::1]").as_deref(), Some("2001:db8::1"));
    assert_eq!(host("::1").as_deref(), Some("::1"));
}

/* ---- phase 5: data correctness ---- */

/// `sessions__delete_expired` must compare the INTEGER `expires_at`
/// against `unixepoch()` — the old `datetime('now')` comparison was
/// always true under SQLite type ordering and wiped live sessions.
#[test]
fn live_session_survives_sweep() {
    let (_db, c) = migrated_db("sess-sweep");
    let live = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 43200;
    insert_session(&c, "live", live);
    insert_session(&c, "dead", 1); // past epoch → sweepable
    insert_session(&c, "immortal", 0); // expires_at=0 → never expires
    c.execute(rpress::sql!("sessions__delete_expired"), []).unwrap();
    assert_eq!(session_count(&c), 2, "live + never-expiring sessions must survive");
    let live_left: i64 = c
        .query_row("SELECT COUNT(*) FROM sessions WHERE token='live'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(live_left, 1);
}

/// `save_term` slug rename cascades into `post_terms` in the same
/// transaction — terms.slug and every assignment move together.
#[test]
fn save_term_rename_cascades_post_terms() {
    let (db, c) = migrated_db("term-rename");
    let id = rpress::db::terms::save_term(&db, "category", None, "News", "news", "", None).unwrap();
    c.execute(
        "INSERT INTO post_terms(post_id, taxonomy, slug, name) VALUES(1,'category','news','News')",
        [],
    )
    .unwrap();
    rpress::db::terms::save_term(&db, "category", Some(id), "Announcements", "announce", "", None).unwrap();
    let term_slug: String = c
        .query_row("SELECT slug FROM terms WHERE id=?1", [id], |r| r.get(0))
        .unwrap();
    assert_eq!(term_slug, "announce");
    let (pt_slug, pt_name): (String, String) = c
        .query_row(
            "SELECT slug, name FROM post_terms WHERE post_id=1 AND taxonomy='category'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!((pt_slug.as_str(), pt_name.as_str()), ("announce", "Announcements"));
}

/// Scheduled publishing stores `published_at` in UTC; `effective_status_at`
/// compares against an injected UTC `now` so the test needs no clock.
#[test]
fn scheduled_post_flips_on_utc_now() {
    use rpress::db::posts::effective_status_at as es;
    assert_eq!(es("publish", Some("2099-01-01 00:00:00"), "2026-01-01 00:00:00"), "future");
    assert_eq!(es("publish", Some("2025-12-31 23:59:59"), "2026-01-01 00:00:00"), "publish");
    assert_eq!(es("future", Some("2025-01-01 00:00:00"), "2026-01-01 00:00:00"), "publish");
    assert_eq!(es("draft", Some("2099-01-01 00:00:00"), "2026-01-01 00:00:00"), "draft");
    assert_eq!(es("publish", None, "2026-01-01 00:00:00"), "publish");
}

/// Site-local wall → UTC → site-local round-trips through the host
/// `date` binary (Europe/Paris is UTC+2 in June, UTC+1 in January).
#[test]
fn local_utc_roundtrip_via_zone() {
    let utc = rpress::db::misc::local_to_utc("2025-06-15 14:00:00", "Europe/Paris");
    assert_eq!(utc.as_deref(), Some("2025-06-15 12:00:00"), "CEST is UTC+2");
    let back = rpress::db::misc::utc_to_local("2025-01-15 12:00:00", "Europe/Paris");
    assert_eq!(back.as_deref(), Some("2025-01-15 13:00:00"), "CET is UTC+1");
    assert_eq!(
        rpress::db::misc::local_to_utc("2025-06-15 14:00:00", "UTC").as_deref(),
        Some("2025-06-15 14:00:00")
    );
}

/// `save_upload` rejects bytes whose sniffed magic family contradicts the
/// extension family (PNG payload in a .jpg).
#[test]
fn save_upload_rejects_mismatched_magic() {
    let (db, _c) = migrated_db("upload-magic");
    let uploads = db.parent().unwrap().join("uploads");
    let png = b"\x89PNG\r\n\x1a\nfiller-filler-filler";
    let err = rpress::db::media::save_upload(&db, &uploads, "evil.jpg", png, "", "t")
        .expect_err("PNG bytes in .jpg must be rejected");
    assert!(err.to_string().contains("does not match"), "{err}");
    // A correctly-labeled PNG still saves.
    let id = rpress::db::media::save_upload(&db, &uploads, "ok.png", png, "", "t").unwrap();
    assert!(id > 0);
}

/// A stale `.snapshot.db` from a crashed backup must not brick the next
/// `run_backup` — VACUUM INTO refuses an existing target.
#[test]
fn backup_survives_stale_snapshot() {
    let (db, _c) = migrated_db("backup-stale");
    let work = db.parent().unwrap().to_path_buf();
    let backups = work.join("backups");
    std::fs::create_dir_all(&backups).unwrap();
    std::fs::write(backups.join(".snapshot.db"), b"stale").unwrap();
    let archive = rpress::backup::run_backup_db(&work, &db, Path::new(""), 3)
        .expect("stale snapshot must be removed, not fatal");
    assert!(archive.is_file());
}

/// `unix_from_civil` is the exact inverse of `civil`-based `now_sql`
/// formatting — 2025-01-01T00:00:00Z = 1735689600.
#[test]
fn unix_from_civil_known_epoch() {
    assert_eq!(
        rpress::db::misc::unix_from_civil("2025-01-01 00:00:00"),
        Some(1735689600)
    );
}

// ---------------------------------------------------------------------------
// Phase 4: import & dump correctness
// ---------------------------------------------------------------------------

/// The `wp_42426_*` prefixed dump imports the tables the old canonicalize
/// map silently dropped — termmeta (→ pll_string_map), rank_math
/// redirections, glsr_ratings + glsr_assigned_posts (→ review_assignments)
/// — plus wp_users → author_login post_meta.
#[test]
fn prefixed_dump_imports_all_previously_dropped_tables() {
    let (db, c) = migrated_db("prefixed-all");
    drop(c); // no live conn (or its WAL) may sit beside db_path at rename
    let d = db.parent().unwrap().to_path_buf();
    let dump = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/prefixed_dump.sql");
    let report = d.join("report.txt");
    rpress::import::run(&dump, &db, &report, true).expect("import");

    let c = rusqlite::Connection::open(&db).unwrap();
    // termmeta arm → `_pll_strings_translations` harvested.
    let pll: String = c
        .query_row("SELECT value FROM settings WHERE key='pll_string_map'", [], |r| r.get(0))
        .expect("pll_string_map missing — wp_termmeta arm dropped the row");
    assert!(pll.contains("Top text"), "pll_string_map: {pll}");
    // rank_math_redirections arm → redirects row.
    let n: i64 = c
        .query_row("SELECT COUNT(*) FROM redirects WHERE old_slug='/old' AND new_url='/new'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1, "rank_math redirection missing");
    // glsr_ratings + glsr_assigned_posts arms → review_assignments row.
    let n: i64 = c
        .query_row("SELECT COUNT(*) FROM review_assignments WHERE review_id=100 AND post_id=100", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1, "glsr assignment missing");
    // sanity: the rest of the prefixed import still works.
    let n: i64 = c
        .query_row("SELECT COUNT(*) FROM post_terms WHERE slug='cat-a'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    let n: i64 = c
        .query_row("SELECT COUNT(*) FROM comments WHERE wp_id=1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 1);
    // staged file is gone after a successful import.
    assert!(!d.join("rpress.db.import").exists(), "stale staging file left");
}

/// `wp_users` rows stream into the users map and land as the `author_login`
/// post_meta chrome/archive read for bylines.
#[test]
fn prefixed_dump_writes_author_login_meta() {
    let (db, c) = migrated_db("prefixed-author");
    drop(c); // no live conn (or its WAL) may sit beside db_path at rename
    let dump = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/prefixed_dump.sql");
    rpress::import::run(&dump, &db, &db.parent().unwrap().join("report.txt"), true).unwrap();
    let c = rusqlite::Connection::open(&db).unwrap();
    let name: String = c
        .query_row(
            "SELECT meta_value FROM post_meta WHERE post_id=100 AND meta_key='author_login'",
            [],
            |r| r.get(0),
        )
        .expect("author_login missing — wp_users arm dropped the row");
    assert_eq!(name, "Admin");
}

/// A `;` followed by a `-- comment` on the same line must not swallow the
/// statement's following rows — and a `/* … */`-prefixed INSERT must parse.
#[test]
fn statement_comments_dont_swallow_or_bail() {
    let d = dir("stmt-comments");
    let p = write_dump(
        &d,
        b"/* generated by phpMyAdmin */\n\
          INSERT INTO `t1` VALUES (1,'a'),(2,'b'); -- trailing comment\n\
          /* comment */ INSERT INTO `t1` VALUES (3,'c');\n",
    );
    let mut got = Vec::new();
    rpress::sqldump::rows(&p, &["t1"], |_, vals| {
        got.push(vals.iter().filter_map(|v| v.to_string_lossy()).collect::<Vec<_>>());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        got,
        vec![vec!["1", "a"], vec!["2", "b"], vec!["3", "c"]],
        "comment-adjacent statements mis-parsed"
    );
}

/// `SET NAMES latin1` dumps transcode cp1252 bytes in unintroduced
/// strings; an undeclared non-UTF-8 byte bails loudly (no U+FFFD decay).
#[test]
fn latin1_dump_transcodes_and_bad_utf8_bails() {
    let d = dir("latin1");
    let p = write_dump(
        &d,
        b"/*!40101 SET NAMES latin1 */;\n\
          INSERT INTO `t1` VALUES ('c\xe9 ph\x93');\n",
    );
    let mut got = Vec::new();
    rpress::sqldump::rows(&p, &["t1"], |_, vals| {
        got.push(vals[0].to_string_lossy().unwrap());
        Ok(())
    })
    .unwrap();
    assert_eq!(got, vec!["c\u{e9} ph\u{201c}".to_string()]);

    let bad = write_dump(&d, b"INSERT INTO `t1` VALUES ('\x93');\n");
    let err = rpress::sqldump::rows(&bad, &["t1"], |_, _| Ok(())).unwrap_err();
    assert!(err.to_string().contains("UTF-8"), "expected loud bail, got: {err}");
}

/// A killed import leaves the prior database intact: writes stage into
/// `<db>.import` and the file is only renamed over `db_path` on success.
#[test]
fn failed_import_leaves_prior_db_intact() {
    let (db, marker_conn) = migrated_db("intact");
    let d = db.parent().unwrap().to_path_buf();
    marker_conn
        .execute(
            "INSERT INTO settings(key,value) VALUES('marker','alive')",
            [],
        )
        .unwrap();
    drop(marker_conn);
    let before = std::fs::read(&db).unwrap();

    // Failing import: valid INSERTs for a wanted table then a REPLACE
    // (parse bails inside the dump pass).
    let dump = write_dump(
        &d,
        b"INSERT INTO `wp_terms` VALUES (1,'A','a',0);\nREPLACE INTO `wp_posts` VALUES (1,'x');\n",
    );
    let err = rpress::import::run(&dump, &db, &d.join("report.txt"), true);
    assert!(err.is_err(), "malformed dump must fail");
    assert_eq!(std::fs::read(&db).unwrap(), before, "db_path bytes changed");

    // Stale `.import` staging file is cleaned on the next run's entry.
    std::fs::write(d.join("rpress.db.import"), b"stale").unwrap();
    let err = rpress::import::run(&dump, &db, &d.join("report.txt"), true);
    assert!(err.is_err());
    assert_eq!(std::fs::read(&db).unwrap(), before, "db_path bytes changed");

    // A subsequent healthy import publishes over db_path.
    let good = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/prefixed_dump.sql");
    rpress::import::run(&good, &db, &d.join("report.txt"), true).expect("import after failures");
    let c = rusqlite::Connection::open(&db).unwrap();
    let n: i64 = c.query_row("SELECT COUNT(*) FROM posts", [], |r| r.get(0)).unwrap();
    assert!(n >= 1);
    assert!(!d.join("rpress.db.import").exists());
}

/// A short INSERT row bails with table context instead of panicking.
#[test]
fn short_insert_row_bails_with_table_context() {
    let d = dir("short-row");
    let dump = write_dump(&d, b"INSERT INTO `wp_posts` VALUES (1,2);\n");
    let err = rpress::import::run(&dump, &d.join("db"), &d.join("report.txt"), true)
        .unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("wp_posts") && msg.contains("too short"), "unhelpful error: {msg}");
}

// ---------------------------------------------------------------------------
// Phase 8: N-language taxonomy resolution + remaining regression coverage
// ---------------------------------------------------------------------------

/// Seed a site with the given languages on the traveltour theme and a
/// taxonomy-map blob; returns (db_path, conn, theme).
fn three_lang_site(name: &str, languages: &str, map_json: &str) -> (PathBuf, rusqlite::Connection) {
    let (db, c) = migrated_db(name);
    c.execute(
        "INSERT INTO settings(key,value) VALUES
         ('stylesheet','traveltour'),
         ('languages',?1),
         ('primary_lang','fr'),
         ('plugin:theme-traveltour:taxonomy_map',?2)",
        rusqlite::params![languages, map_json],
    )
    .unwrap();
    (db, c)
}

/// 3-language site (fr primary, es + de secondary): `lang_roots` per-lang
/// category roots resolve archive lang for BOTH secondary langs — the es
/// root chains to "es", the de root chains to "de", and a `-de`-suffixed
/// slug outside the map still lands on "de" via the suffix heuristic.
/// No silent fallback to primary on the third tree.
#[test]
fn phase08_three_lang_cat_resolution() {
    let map = serde_json::json!({
        "cat_edges": {"guia-child": "guia-root", "guide-child": "guide-root"},
        "lang_roots": {"es": ["guia-root"], "de": ["guide-root"]}
    })
    .to_string();
    let (_db, c) = three_lang_site("nlang3", "fr,es,de", &map);
    let theme = rpress::plugins::theme_for_site(&c);

    let lang = |slug: &str| rpress::plugins::theme_cat_lang(&c, theme, slug, "fr");
    assert_eq!(lang("guia-child").as_deref(), Some("es"), "es root chain");
    assert_eq!(lang("guide-child").as_deref(), Some("de"), "de root chain — no primary fallback");
    assert_eq!(lang("random-slug-de").as_deref(), Some("de"), "registered -de suffix");
    assert_eq!(lang("random-slug-es").as_deref(), Some("es"), "registered -es suffix");
    assert_eq!(lang("thing-xx").as_deref(), Some("fr"), "unregistered suffix → primary");
    assert_eq!(lang("plain-cat").as_deref(), Some("fr"), "no signal → primary");
}

/// The legacy `es_roots`/`es_roots_lang` blob shape keeps working on
/// 2-lang sites; when `lang_roots` is also present it wins.
#[test]
fn phase08_es_roots_fallback_and_lang_roots_wins() {
    let map = serde_json::json!({
        "cat_edges": {"guia-child": "guia-root", "guide-child": "guide-root"},
        "es_roots": ["guia-root"],
        "es_roots_lang": "es"
    })
    .to_string();
    let (_db, c) = three_lang_site("nlang2", "fr,es", &map);
    let theme = rpress::plugins::theme_for_site(&c);
    let lang = |slug: &str| rpress::plugins::theme_cat_lang(&c, theme, slug, "fr");
    assert_eq!(lang("guia-child").as_deref(), Some("es"), "legacy es_roots chain");
    assert_eq!(lang("some-cat-es").as_deref(), Some("es"), "es suffix on 2-lang site");

    // Both shapes present: lang_roots wins (es chains resolve through it).
    let map = serde_json::json!({
        "cat_edges": {"guia-child": "guia-root", "guide-child": "guide-root"},
        "es_roots": ["guide-root"],
        "es_roots_lang": "es",
        "lang_roots": {"de": ["guide-root"], "es": ["guia-root"]}
    })
    .to_string();
    let (_db2, c2) = three_lang_site("nlang-both", "fr,es,de", &map);
    let theme2 = rpress::plugins::theme_for_site(&c2);
    let lang2 = |slug: &str| rpress::plugins::theme_cat_lang(&c2, theme2, slug, "fr");
    assert_eq!(lang2("guide-child").as_deref(), Some("de"), "lang_roots wins over es_roots");
    assert_eq!(lang2("guia-child").as_deref(), Some("es"));
}

/// `localized_taxonomy` consults `tax_lang` for any registered secondary
/// lang: a registered twin resolves, an unregistered lang keeps the
/// primary name — no hardcoded "es".
#[test]
fn phase08_localized_taxonomy_nlang() {
    let (_db, c) = three_lang_site("nlang-tax", "fr,es,de", "{}");
    let theme = rpress::plugins::theme_for_site(&c);
    // traveltour data: ("style","estilo") with estilo→es.
    assert_eq!(rpress::plugins::localized_taxonomy(theme, "style", "es", "fr"), "estilo");
    assert_eq!(rpress::plugins::localized_taxonomy(theme, "style", "fr", "fr"), "style");
    // "de" has no twin registered for "style" → primary name, not silent
    // es fallback.
    assert_eq!(rpress::plugins::localized_taxonomy(theme, "style", "de", "fr"), "style");
}

/// Non-object `tourmaster-tour-option` payloads (a JSON scalar where the
/// blob should be a dict) bail with context instead of silently dropping
/// the save's fields.
#[test]
fn phase08_tour_option_requires_object() {
    let (db, _c) = migrated_db("tour-opt-guard");
    let f: rpress::db::posts::PostForm = serde_urlencoded::from_str(
        "kind=tour&title=T&slug=t&lang=fr&status=publish&tourmaster-tour-option=5",
    )
    .unwrap();
    let err = rpress::db::posts::save_post(&db, &f).unwrap_err();
    let msg = format!("{err:#}");
    assert!(
        msg.contains("not an object"),
        "scalar tour-option blob must bail loudly: {msg}"
    );
}

/// `clone_post` duplicates the adapter-owned `tours` row too — every tour
/// column, not just the post/meta/terms shared tables.
#[test]
fn phase08_clone_post_copies_tour_columns() {
    let (db, c) = migrated_db("clone-tour");
    c.execute(
        "INSERT INTO posts(id,slug,lang,title,status,kind) VALUES(1,'t','fr','T','publish','tour')",
        [],
    )
    .unwrap();
    c.execute(
        "INSERT INTO tours(post_id,duration_days,departure_dates_json,price_range,max_people,tour_time,gallery_json,rating,view_count)
         VALUES(1,7,'[\"2026-01-01\"]','100-200',12,'9h','[1,2]',4.5,42)",
        [],
    )
    .unwrap();
    let new_id = rpress::db::posts::clone_post(&db, 1, false).unwrap();
    assert_ne!(new_id, 1);
    let row: (i64, String, String, i64, String, String, f64, i64) = c
        .query_row(
            "SELECT duration_days,departure_dates_json,price_range,max_people,tour_time,gallery_json,rating,view_count FROM tours WHERE post_id=?1",
            [new_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .expect("clone must copy the tours row");
    assert_eq!(
        row,
        (7, "[\"2026-01-01\"]".to_string(), "100-200".to_string(), 12, "9h".to_string(), "[1,2]".to_string(), 4.5, 42)
    );
}
/// `PostForm.cat` decoding contract (audit L — this test was print-only):
/// the checkbox list joins selections with \u{1f} inside a single `cat`
/// field; `de_cat` splits it back into slugs.
#[test]
fn cat_seq() {
    // Single cat field carrying US-separated slug:name pairs — the wire
    // format the post form actually emits.
    let f = serde_urlencoded::from_str::<rpress::db::posts::PostForm>(
        "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat=a%3Ab%1Fc%3Ad&tags=z",
    )
    .expect("US-joined cat parses");
    assert_eq!(f.cat, vec!["a:b".to_string(), "c:d".to_string()]);

    // PHP-style `cat[]` keys don't exist in our forms; they fall into the
    // `extra` flatten map, leaving `cat` empty rather than erroring.
    let f = serde_urlencoded::from_str::<rpress::db::posts::PostForm>(
        "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat[]=a:b&cat[]=c:d&tags=z",
    )
    .expect("cat[] form parses");
    assert!(f.cat.is_empty());
    assert_eq!(f.extra.get("cat[]").map(|s| s.as_str()), Some("c:d"));

    // Repeated bare `cat` keys are a duplicate field — rejected, not a
    // silent last-wins collapse.
    let r = serde_urlencoded::from_str::<rpress::db::posts::PostForm>(
        "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat=a:b&cat=c:d&tags=z",
    );
    assert!(r.is_err(), "duplicate cat keys must error, not collapse");
}
