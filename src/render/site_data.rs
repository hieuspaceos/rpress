//! `site` template data — posts/tours/widgets JSON materialized once per
//! render (moved out of render.rs — pure move, no behavior change).

#[allow(unused_imports)]
use super::*;


/// Site-wide template data (posts/tours/widgets JSON) keyed by
/// (site_url, lang) — the probe measured `site_data` at ~55% of a
/// cache-miss render, so it earns a cache. Caches are process-global under
/// multi-site serve; site_url (derived from the DB's siteurl setting per
/// site) disambiguates. Invalidated by `invalidate_dependents` (post-level
/// saves) and `bust_cache_dir` (everything else) — same funnels as ENVS.
static SITE_DATA: std::sync::LazyLock<parking_lot::RwLock<HashMap<(String, String), JVal>>> =
    std::sync::LazyLock::new(parking_lot::RwLock::default);

/// Drop all cached site_data — call from every invalidation path.
pub(crate) fn site_data_bust() {
    SITE_DATA.write().clear();
}

/// `site_data` behind the (site_url, lang)-keyed cache; compute-on-miss.
pub(crate) fn site_data_cached(
    conn: &Connection,
    lang: &str,
    site_url: &str,
    tour_base: &str,
) -> Result<JVal> {
    let key = (site_url.to_string(), lang.to_string());
    if let Some(v) = SITE_DATA.read().get(&key) {
        return Ok(v.clone());
    }
    let v = site_data(conn, lang, site_url, tour_base)?;
    SITE_DATA.write().entry(key).or_insert_with(|| v.clone());
    Ok(v)
}

/// Site-wide data exposed to element templates as `site` so dynamic
/// elements (blog, tour, testimonial, tour_search, custom-menu…) can render
/// without issuing their own queries. Built once per render call.
pub(crate) fn site_data(
    conn: &Connection,
    lang: &str,
    site_url: &str,
    tour_base: &str,
) -> Result<JVal> {
    let lang_cfg = crate::lang::config(conn);
    let prefix = crate::lang::url_prefix(&lang_cfg, lang);
    let url = |kind: &str, slug: &str| page_url(conn, site_url, lang, kind, slug, tour_base);
    // Featured images resolve one point-lookup at a time (lazy media_map) —
    // the whole media table is never materialized here. Lookups cache in
    // `seen` so repeated fids stay one query.
    let seen = std::cell::RefCell::new(HashMap::<i64, Option<(String, String)>>::new());
    let media_row = |mid: i64| -> Option<(String, String)> {
        seen.borrow_mut()
            .entry(mid)
            .or_insert_with(|| media_lookup(conn, mid))
            .clone()
    };
    let img = |mid: Option<i64>| -> Value {
        mid.and_then(&media_row)
            .map(|(p, _)| json!(format!("{site_url}/wp-content/uploads/{p}")))
            .unwrap_or(Value::Null)
    };

    // posts (kind='post') with category slugs
    let mut posts = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, slug, title, COALESCE(excerpt,''), featured_media_id, \
             COALESCE(published_at,'') FROM posts \
             WHERE kind='post' AND lang=?1 AND status='publish' \
             ORDER BY published_at DESC",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        for row in rows {
            let (id, slug, title, excerpt, fid, date) = row?;
            let mut cats = Vec::new();
            let mut cstmt = conn.prepare(
                "SELECT slug, name FROM post_terms WHERE post_id=?1 AND taxonomy='category'",
            )?;
            let crows = cstmt.query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            for c in crows.flatten() {
                cats.push(json!({"slug": c.0, "name": c.1}));
            }
            posts.push(json!({
                "id": id, "slug": slug, "title": title, "excerpt": excerpt,
                "date": date, "url": url("post", &slug), "image": img(fid),
                "categories": cats,
            }));
        }
    }
    // tours feed moved behind the owning adapter's `site_data_feeds`
    // (phase 03); merged into the site map below.

    // testimonials: site-review posts — a core feed (no registered adapter).
    let reviews = site_reviews(conn, site_url)?;

    // terms grouped by taxonomy
    let mut terms: Map<String, Value> = Map::new();
    {
        let mut stmt = conn.prepare(
            "SELECT taxonomy, slug, name FROM terms ORDER BY taxonomy, name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        })?;
        for row in rows {
            let (tax, slug, name) = row?;
            terms
                .entry(tax)
                .or_insert_with(|| Value::Array(vec![]))
                .as_array_mut()
                .unwrap()
                .push(json!({"slug": slug, "name": name}));
        }
    }

    // menus keyed by name, id and wp_term_id; items parented into trees
    let mut menus: Map<String, Value> = Map::new();
    {
        // WP stores a blank nav_label when the item reuses the target's
        // title — resolve it here: post_type → posts.title, taxonomy →
        // terms.name, otherwise stay blank (never a bogus "menu item").
        let mut istmt = conn.prepare(
            "SELECT m.id, m.wp_term_id, m.name, m.lang, m.location, i.id, i.parent_id, i.ord, \
             COALESCE(NULLIF(i.label,''), \
               CASE i.item_type \
                 WHEN 'post_type' THEN (SELECT title FROM posts WHERE id=i.object_id AND lang=m.lang LIMIT 1) \
                 WHEN 'post_type' THEN (SELECT title FROM posts WHERE id=i.object_id LIMIT 1) \
                 WHEN 'taxonomy' THEN (SELECT name FROM terms WHERE id=i.object_id LIMIT 1) \
               END, i.label), \
             COALESCE(i.url,''), i.object_id, i.item_type, COALESCE(i.classes,''), \
             COALESCE(i.title_attr,''), COALESCE(i.mega,'') FROM menus m \
             JOIN menu_items i ON i.menu_id=m.id ORDER BY i.ord",
        )?;
        let rows = istmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<i64>>(7)?,
                r.get::<_, Option<String>>(8)?,
                r.get::<_, String>(9)?,
                r.get::<_, Option<i64>>(10)?,
                r.get::<_, Option<String>>(11)?,
                r.get::<_, String>(12)?,
                r.get::<_, String>(13)?,
                r.get::<_, String>(14)?,
            ))
        })?;
        /* (menu db id, wp term id, name, lang, location, items) — lang and
           location drive the render-time menu pick. */
        let mut by_menu: HashMap<i64, (i64, String, Option<String>, Option<String>, Vec<Value>)> =
            HashMap::new();
        let mut menu_meta: Vec<(i64, Option<String>, Option<String>)> = Vec::new(); // (id, lang, location)
        for row in rows {
            let (mid, wpid, name, mlang, mloc, iid, parent, ord, label, iurl, oid, itype, cls, tattr, mega) =
                row?;
            let entry = by_menu
                .entry(mid)
                .or_insert_with(|| (wpid, name, mlang.clone(), mloc.clone(), Vec::new()));
            entry.4.push(json!({
                "id": iid, "parent_id": parent.unwrap_or(0), "ord": ord,
                "title": label.unwrap_or_default(), "url": iurl,
                "object_id": oid, "item_type": itype.unwrap_or_default(),
                "extra_class": cls, "title_attr": tattr, "mega": mega,
                "children": [],
            }));
        }
        let aliases = menu_aliases(conn, site_url);
        for (mid, (wpid, name, mlang, mloc, flat)) in by_menu {
            let tree = build_menu_tree(flat);
            /* Resolve URLs + drop dead items into every stored copy — the
               tree is per-menu not per-location, and resolution is already
               scoped to `lang` (the site_data cache key carries it). */
            let tree = resolve_tree(conn, tree, site_url, tour_base, lang, &aliases);
            let v = json!(tree);
            menus.insert(name.clone(), v.clone());
            menus.insert(mid.to_string(), v.clone());
            menus.insert(format!("wp:{wpid}"), v);
            menu_meta.push((mid, mlang, mloc));
        }
        /* Render picks: theme_locations (location,lang)→menu first —
           WP's nav_menu_locations imported verbatim; then menus.lang
           match; then first menu. menu_footer tries footer-ish locations. */
        let pick = |locs: &[&str]| -> Option<i64> {
            let mut st = conn
                .prepare("SELECT menu_id FROM theme_locations WHERE location=?1 AND lang=?2")
                .ok()?;
            for loc in locs {
                if let Ok(Some(mid)) = st.query_row(rusqlite::params![loc, lang], |r| r.get(0)) {
                    return Some(mid);
                }
            }
            None
        };
        let _ = &menu_meta; // kept for the lang fallback below
        let header_locs = ["main_menu", "main", "primary", "header"];
        let footer_locs = ["footer", "footer_menu", "secondary"];
        let mut chosen = pick(&header_locs);
        if chosen.is_none() {
            if let Ok(mut s) = conn.prepare(
                "SELECT id FROM menus WHERE COALESCE(lang,'')=?1 LIMIT 1",
            ) {
                chosen = s.query_row([lang], |r| r.get::<_, i64>(0)).ok();
            }
        }
        /* `*_html` pre-renders to WP <li> markup so base templates stay
           structural (minijinja can't recurse cleanly). */
        if let Some(v) = chosen.and_then(|mid| menus.get(&mid.to_string()).cloned()) {
            if let Some(items) = v.as_array() {
                menus.insert("menu_main_html".into(), json!(menu_li(items, site_url)));
                menus.insert("menu_mobile_html".into(), json!(menu_li_mobile(items, site_url)));
                menus.insert("menu_main_tevily_html".into(), json!(menu_li_tevily(items, site_url)));
                /* splitted-menu header: WP traveltour renders the logo as
                   the middle <li> — first ceil(N/2) top-level items
                   sit BEFORE it. Templates read menu_main_left_html /
                   menu_main_right_html for the split. */
                let half = items.len().div_ceil(2);
                menus.insert(
                    "menu_main_left_html".into(),
                    json!(menu_li(&items[..half], site_url)),
                );
                menus.insert(
                    "menu_main_right_html".into(),
                    json!(menu_li(&items[half..], site_url)),
                );
            }
            menus.insert("menu_main".into(), v);
        }
        if let Some(v) = pick(&footer_locs).and_then(|mid| menus.get(&mid.to_string()).cloned()) {
            if let Some(items) = v.as_array() {
                menus.insert("menu_footer_html".into(), json!(menu_li(items, site_url)));
            }
            menus.insert("menu_footer".into(), v);
        }
    }

    // tourmaster option-derived keys (search_url/search_months) moved into
    // the tourmaster adapter's `site_data_feeds` with the tours feed.

    // Adapter-owned feeds (tours list, search options, …) merge under
    // their declared keys; the kind URL base resolves through KindBases.
    let bases = crate::admin::plugins::resolve_bases(tour_base);
    let mut extra: Map<String, Value> = Map::new();
    for adapter in crate::admin::plugins::adapters() {
        let kind_base = adapter
            .kinds()
            .first()
            .and_then(|k| crate::admin::plugins::base_for(&bases, k.kind));
        for (k, v) in adapter.site_data_feeds(conn, lang, site_url, kind_base) {
            extra.insert(k, v);
        }
    }

    // pages for page-list
    let mut pages = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, slug, title, COALESCE(parent_id,0), COALESCE(menu_order,0) \
             FROM posts WHERE kind='page' AND lang=?1 AND status='publish' \
             ORDER BY menu_order, title",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })?;
        for row in rows {
            let (id, slug, title, parent_id, ord) = row?;
            pages.push(json!({
                "id": id, "slug": slug, "title": title,
                "url": url("page", &slug), "parent_id": parent_id, "order": ord,
            }));
        }
    }

    let mut map = serde_json::json!({
        "posts": posts,
        "reviews": reviews,
        "terms": terms,
        "menus": menus,
        "pages": pages,
        "lang": lang,
        "url_prefix": prefix,
        "site_url": site_url,
        "tour_base_url": format!("{site_url}{prefix}/{tour_base}/"),
        "chrome": chrome_ctx(conn, site_url, lang),
    });
    if let Some(obj) = map.as_object_mut() {
        for (k, v) in extra {
            obj.insert(k, v);
        }
    }
    Ok(JVal::from_serialize(map))
}

/// testimonials: site-review posts; reviewer name from _submitted meta.
/// Core feed — no registered adapter owns the `site-review` kind.
fn site_reviews(conn: &Connection, site_url: &str) -> Result<Vec<Value>> {
    let img = |mid: Option<i64>| -> Value {
        mid.and_then(|m| media_lookup(conn, m))
            .map(|(p, _)| json!(format!("{site_url}/wp-content/uploads/{p}")))
            .unwrap_or(Value::Null)
    };
    let mut reviews = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, title, COALESCE(body_json,''), COALESCE(excerpt,''), featured_media_id \
         FROM posts WHERE kind='site-review' AND status='publish' \
         ORDER BY published_at DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<i64>>(4)?,
        ))
    })?;
    for row in rows {
        let (id, title, body_json, excerpt, fid) = row?;
        let mut name = title.clone();
        let mut rating: Option<i64> = None;
        let submitted: Option<String> = conn
            .query_row(
                "SELECT meta_value FROM post_meta WHERE post_id=?1 AND meta_key='_submitted' LIMIT 1",
                [id],
                |r| r.get(0),
            )
            .ok();
        if let Some(raw) = submitted {
            if let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) {
                if let Some(n) = v.get("name").and_then(|x| x.as_str()) {
                    if !n.is_empty() {
                        name = n.to_string();
                    }
                }
                rating = v
                    .get("rating")
                    .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())));
            }
        }
        // review text: body_json is a fragment of text-box elements
        let mut content = String::new();
        if let Ok(bj) = serde_json::from_str::<Value>(&body_json) {
            collect_textbox(&bj, &mut content);
        }
        reviews.push(json!({
            "id": id, "name": name, "content": content, "excerpt": excerpt,
            "rating": rating, "image": img(fid),
        }));
    }
    Ok(reviews)
}

/// Pull text-box contents out of a body_json fragment (for review posts).
fn collect_textbox(v: &Value, out: &mut String) {
    match v {
        Value::Array(a) => a.iter().for_each(|x| collect_textbox(x, out)),
        Value::Object(m) => {
            if m.get("type").and_then(|t| t.as_str()) == Some("text-box") {
                if let Some(c) = m
                    .get("value")
                    .and_then(|v| v.get("content"))
                    .and_then(|c| c.as_str())
                {
                    if !out.is_empty() {
                        out.push_str("<br/>");
                    }
                    out.push_str(c);
                }
            }
            for x in m.values() {
                collect_textbox(x, out);
            }
        }
        _ => {}
    }
}


/// Nest flat menu items (with parent_id) into a children tree.
fn build_menu_tree(flat: Vec<Value>) -> Vec<Value> {
    let mut nodes: HashMap<i64, Value> = HashMap::new();
    let mut order = Vec::new();
    for it in flat {
        let id = it.get("id").and_then(|x| x.as_i64()).unwrap_or(0);
        order.push(id);
        nodes.insert(id, it);
    }
    fn attach(id: i64, nodes: &mut HashMap<i64, Value>, seen: &mut Vec<i64>) -> Option<Value> {
        if seen.contains(&id) {
            return None;
        }
        seen.push(id);
        let mut node = nodes.get(&id)?.clone();
        let mut children = Vec::new();
        let mut child_ids: Vec<i64> = nodes
            .iter()
            .filter(|(_, n)| n.get("parent_id").and_then(|p| p.as_i64()) == Some(id))
            .map(|(cid, _)| *cid)
            .collect();
        /* WP menu order is `ord` — HashMap iteration is unordered. */
        child_ids.sort_by_key(|cid| {
            nodes
                .get(cid)
                .and_then(|n| n.get("ord").and_then(|o| o.as_i64()))
                .unwrap_or(0)
        });
        for cid in child_ids {
            if let Some(c) = attach(cid, nodes, seen) {
                children.push(c);
            }
        }
        node.as_object_mut()
            .unwrap()
            .insert("children".into(), Value::Array(children));
        Some(node)
    }
    let mut seen = Vec::new();
    let mut roots = Vec::new();
    for id in order {
        /* Orphaned item (parent absent from this menu — trashed on origin
           or filtered at import): WP drops the subtree, never promotes
           it to root. Promoting inflates the top-level <li> count vs
           the origin nav. */
        let is_root = nodes
            .get(&id)
            .and_then(|n| n.get("parent_id").and_then(|p| p.as_i64()))
            .map(|p| p == 0)
            .unwrap_or(true);
        if is_root {
            if let Some(n) = attach(id, &mut nodes, &mut seen) {
                roots.push(n);
            }
        }
    }
    roots
}


/// Alias hosts for `strip_origin`: the imported origin (`settings.site_url`
/// is what generated URLs carry) plus `settings.url_aliases` CSV — the
/// served `site_url` arg is always matched too.
fn menu_aliases(conn: &Connection, site_url: &str) -> String {
    /* Hosts whose absolute URLs still mean "this site": the imported
       legacy_origin (WP's own domain at dump time — what topbar/menu
       payloads carry), the current settings.site_url, plus any
       url_aliases CSV extras. `strip_origin` re-anchors them all on the
       live site_url. */
    let get = |k: &str| -> String {
        conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default()
    };
    let served = site_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or("");
    [get("legacy_origin"), get("site_url"), get("url_aliases"), served.to_string()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

/// Walk a built menu tree: resolve each item's URL via `menu_url`, dropping
/// items whose resolution returns `None` — WP hides dead menu items, so
/// their subtrees die with them (never emit `href="#"`).
fn resolve_tree(
    conn: &Connection,
    items: Vec<Value>,
    site_url: &str,
    tour_base: &str,
    lang: &str,
    aliases: &str,
) -> Vec<Value> {
    resolve_tree_depth(conn, items, site_url, tour_base, lang, aliases, false)
}

/// `gdlr_core_nav_menu_custom` field lookup — serialized assoc whose
/// `enable-mega-menu` / `mega-menu-section-size` drive the traveltour
/// mega-menu markup.
fn mega_field(mega: &str, key: &str) -> Option<String> {
    let v = crate::php_serialize::unserialize(mega.as_bytes()).ok()?;
    v.get(key).and_then(|x| x.as_str().map(str::to_string))
}

fn resolve_tree_depth(
    conn: &Connection,
    items: Vec<Value>,
    site_url: &str,
    tour_base: &str,
    lang: &str,
    aliases: &str,
    in_mega: bool,
) -> Vec<Value> {
    let mut out = Vec::new();
    for mut it in items {
        let item_type = it.get("item_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let object_id = it.get("object_id").and_then(|v| v.as_i64());
        let raw = it.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        /* Empty label on a taxonomy item = WP "navigation label"
           untouched → the term name renders. */
        if it.get("title").and_then(|v| v.as_str()).unwrap_or("").is_empty()
            && item_type == "taxonomy"
        {
            if let Some(name) = object_id.and_then(|oid| {
                conn.query_row("SELECT name FROM terms WHERE id=?1", [oid], |r| {
                    r.get::<_, String>(0)
                })
                .ok()
            }) {
                it.as_object_mut().unwrap().insert("title".into(), json!(name));
            }
        }
        match crate::render::menu_url::resolve_menu_url(
            conn, item_type.as_str(), object_id, &raw, site_url, tour_base, lang, aliases,
        ) {
            Some(u) => {
                let m = it.as_object_mut().unwrap();
                /* Empty url = `#` container: WP renders a bare <a> (no
                   href). `nohref` tells the template, keeps `url` "" for
                   navx (which never marks a no-href <a> anyway). */
                m.insert("nohref".into(), json!(u.is_empty()));
                m.insert("url".into(), json!(u));
            }
            None => continue, // dead item — subtree dropped wholesale
        }
        /* WP <li> classes the theme + apply_navx both key on:
           menu-item-{id}, menu-item-type-{type}, menu-item-object-{obj},
           menu-item-has-children. obj = kind for posts, taxonomy for
           terms, `custom` otherwise. */
        let obj = match item_type.as_str() {
            "post_type" | "page" | "post" => object_id
                .and_then(|oid| {
                    conn.query_row(
                        "SELECT kind FROM posts WHERE wp_id=?1",
                        [oid],
                        |r| r.get::<_, String>(0),
                    )
                    .ok()
                })
                .unwrap_or_else(|| "post".into()),
            "taxonomy" => object_id
                .and_then(|oid| {
                    conn.query_row("SELECT taxonomy FROM terms WHERE id=?1", [oid], |r| {
                        r.get::<_, String>(0)
                    })
                    .ok()
                })
                .unwrap_or_default(),
            _ => "custom".into(),
        };
        let kids = it
            .get("children")
            .and_then(|c| c.as_array().cloned())
            .unwrap_or_default();
        /* traveltour mega-menu (gdlr_core_nav_menu_custom):
           root with enable-mega-menu=enable  → `traveltour-mega-menu` +
             `<div class="sf-mega sf-mega-full">` wrapping its sub-menu.
           Children inside the mega grid: li[data-size] = the item's own
             mega-menu-section-size (default 60); a child with children
             but no section size gets `traveltour-normal-menu` instead —
             it drops back to a normal flyout.
           `_menu_item_classes` extras append verbatim (menu-tour-max…). */
        let mega = it.get("mega").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mega_root = mega_field(&mega, "enable-mega-menu").as_deref() == Some("enable");
        let kids = resolve_tree_depth(
            conn,
            kids,
            site_url,
            tour_base,
            lang,
            aliases,
            in_mega || mega_root,
        );
        let has_kids = !kids.is_empty();
        let iid = it.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let extra = it
            .get("extra_class")
            .and_then(|v| v.as_str())
            .filter(|c| !c.is_empty())
            .map(|c| format!("{c} "))
            .unwrap_or_default();
        let mut classes = format!(
            "{extra}menu-item-{iid} menu-item-type-{item_type} menu-item-object-{obj} menu-item{}",
            if has_kids { "-has-children" } else { "" },
        );
        let m = it.as_object_mut().unwrap();
        if mega_root {
            classes.push_str(" traveltour-mega-menu");
            m.insert("mega_full".into(), json!(true));
        }
        if in_mega && !mega_root {
            match mega_field(&mega, "mega-menu-section-size") {
                Some(sz) => {
                    m.insert("data_size".into(), json!(sz));
                }
                None => {
                    if has_kids {
                        classes.push_str(" traveltour-normal-menu");
                    } else {
                        m.insert("data_size".into(), json!("60"));
                    }
                }
            }
        } else if !mega_root {
            /* WP traveltour: EVERY non-mega top-level item renders as a
               normal flyout — `traveltour-normal-menu` applies to childless
               items too (the class marks "not a mega grid"; submenu CSS/JS
               hang off it). */
            classes.push_str(" traveltour-normal-menu");
        }
        m.insert("children".into(), Value::Array(kids));
        m.insert("classes".into(), json!(classes));
        out.push(it);
    }
    out
}

/// Render a resolved menu tree to WP nav markup: `<li class="{classes}">
/// <a href>{title}</a>{<ul class="sub-menu">…</ul>}</li>`. `nohref` items
/// emit a bare `<a>`; `title_attr` maps to the anchor's title; the label
/// may itself be HTML (image menu items) and is emitted verbatim.
fn menu_li(items: &[Value], site_url: &str) -> String {
    menu_li_mode(items, true, site_url)
}

/// Mobile variant: WP's mmenu tree never carries mega wrappers,
/// `data-size`, or the mega/normal classes — strip them per item.
fn menu_li_mobile(items: &[Value], site_url: &str) -> String {
    menu_li_mode(items, false, site_url)
}

fn menu_li_mode(items: &[Value], mega: bool, site_url: &str) -> String {
    /* `{{ site_url }}` tokens in menu labels (image items rewritten at
       import) resolve against the serving host — same token expansion
       `render_fragment` applies to widget html. */
    let mut out = String::new();
    for it in items {
        let mut classes = it
            .get("classes")
            .and_then(|v| v.as_str())
            .unwrap_or("menu-item")
            .to_string();
        if !mega {
            classes = classes
                .replace(" traveltour-mega-menu", "")
                .replace(" traveltour-normal-menu", "");
        }
        let url = it.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let title = &it
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .replace("{{ site_url }}", site_url);
        let tattr = it.get("title_attr").and_then(|v| v.as_str()).unwrap_or("");
        let nohref = it.get("nohref").and_then(|v| v.as_bool()).unwrap_or(false);
        let kids = it.get("children").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let dsize = it.get("data_size").and_then(|v| v.as_str()).filter(|_| mega);
        let ds = dsize.map(|s| format!(" data-size=\"{s}\"")).unwrap_or_default();
        out.push_str(&format!("<li class=\"{classes}\"{ds}>"));
        /* WP traveltour marks parent anchors `sf-with-ul-pre` (the
           superfish arrow emitted server-side) on mega-menu roots and
           in-mega containers — without it the dropdown nav renders no
           submenu. Fields set in resolve_tree_depth: `mega_full`,
           `data_size`, or the `traveltour-normal-menu` class. */
        let in_mega_ctx = it
            .get("mega_full")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            || it.get("data_size").is_some()
            || classes.contains("traveltour-normal-menu");
        let sf_class = if !kids.is_empty() && in_mega_ctx {
            " class=\"sf-with-ul-pre\""
        } else {
            ""
        };
        if nohref {
            out.push_str(&format!("<a{sf_class}>{title}</a>"));
        } else {
            let t = if tattr.is_empty() { String::new() } else { format!(" title=\"{tattr}\"") };
            out.push_str(&format!("<a href=\"{url}\"{t}{sf_class}>{title}</a>"));
        }
        if !kids.is_empty() {
            if mega && it.get("mega_full").and_then(|v| v.as_bool()).unwrap_or(false) {
                out.push_str("<div class=\"sf-mega sf-mega-full\"><ul class=\"sub-menu\">");
                out.push_str(&menu_li_mode(&kids, mega, site_url));
                out.push_str("</ul></div>");
            } else {
                out.push_str("<ul class=\"sub-menu\">");
                out.push_str(&menu_li_mode(&kids, mega, site_url));
                out.push_str("</ul>");
            }
        }
        out.push_str("</li>");
    }
    out
}

/// Tevily-shaped <li> markup (`templates/theme/tevily/base_*.html`):
/// submenu ul is `.submenu-inner`, parent links carry a `<span
/// class="caret">`, titles are wrapped in `.menu-title`. Same item
/// pipeline as `menu_li` — classes/urls come from `menu_items` rows.
fn menu_li_tevily(items: &[Value], site_url: &str) -> String {
    let mut out = String::new();
    for it in items {
        let classes = it.get("classes").and_then(|v| v.as_str()).unwrap_or("menu-item");
        let url = it.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let title = &it
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .replace("{{ site_url }}", site_url);
        let nohref = it.get("nohref").and_then(|v| v.as_bool()).unwrap_or(false);
        let kids = it.get("children").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let caret = if kids.is_empty() { "" } else { "<span class=\"caret\"></span>" };
        out.push_str(&format!("<li class=\"{classes}\">"));
        if nohref {
            out.push_str(&format!("<a><span class=\"menu-title\">{title}</span>{caret}</a>"));
        } else {
            out.push_str(&format!("<a href=\"{url}\"><span class=\"menu-title\">{title}</span>{caret}</a>"));
        }
        if !kids.is_empty() {
            out.push_str("<ul class=\"submenu-inner \">");
            out.push_str(&menu_li_tevily(&kids, site_url));
            out.push_str("</ul>");
        }
        out.push_str("</li>");
    }
    out
}

/// Per-site chrome values the base templates read as `site.chrome.*`.
/// Everything here is seeded from import (theme options, plugin payloads)
/// or derived from settings — never a hardcoded aucoeur string.
fn chrome_ctx(conn: &Connection, site_url: &str, lang: &str) -> Value {
    let get = |k: &str| -> String {
        conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get::<_, String>(0))
            .unwrap_or_default()
    };
    let theme: Value = get("plugin:theme-traveltour:general")
        .parse::<serde_json::Value>()
        .unwrap_or(Value::Null);
    // Per-site generated css paths; "" when the site's uploads lack the
    // file → template omits the link. Tourmaster's two files share the
    // carry mechanism (same migrate pass writes all keys).
    let theme_css = get("plugin:theme:css_upload_path");
    let tourmaster_css = get("plugin:tourmaster:css_upload_path");
    let tourmaster_css_global = get("plugin:tourmaster:css_global_upload_path");
    let tourmaster_css_room = get("plugin:tourmaster:css_room_upload_path");
    let tget = |k: &str| -> String {
        theme.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };
    let media_path = |id: &str| -> Option<(String, String)> {
        let wid: i64 = id.parse().ok()?;
        conn.query_row(
            "SELECT path, COALESCE(alt,'') FROM media WHERE wp_id=?1",
            [wid],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .ok()
    };
    // Fallback when the theme option is unset (common on imported sites):
    // first media whose filename contains 'logo', smallest wp_id — the
    // same heuristic the origin's header uses.
    let logo_fallback = || -> Option<(String, String)> {
        conn.query_row(
            "SELECT path, COALESCE(alt,'') FROM media              WHERE LOWER(path) LIKE '%logo%' ORDER BY wp_id LIMIT 1",
            [],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .ok()
    };
    let logo_img = |key: &str, cls: &str| -> Value {
        media_path(&tget(key))
            .or_else(logo_fallback)
            .map(|(p, alt)| {
                json!(format!(
                    "<a class=\"{cls}\" href=\"{site_url}/\" ><img src=\"{site_url}/wp-content/uploads/{p}\" alt=\"{alt}\" srcset=\"{site_url}/wp-content/uploads/{p} 1x, {site_url}/wp-content/uploads/{p} 2x\"  /></a>"
                ))
            })
            .unwrap_or_else(|| json!(""))
    };
    // top-bar HTML payloads carry origin URLs + [gdlr_core_icon]
    // shortcodes: expand shortcodes, then re-anchor own-origin links on
    // the live site_url.
    /* Polylang string map: option strings stored in primary lang pair
       with translations — non-primary pages substitute the translated
       copy verbatim (same post-processing as the primary string). */
    let primary = conn
        .query_row("SELECT value FROM settings WHERE key='primary_lang'", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap_or_default();
    let pll_map: Vec<(String, String)> =
        serde_json::from_str(&get("pll_string_map")).unwrap_or_default();
    let translated = |raw: &str| -> String {
        if lang == primary || pll_map.is_empty() {
            return raw.to_string();
        }
        /* pll_string_map: (primary-copy, translated-copy) pairs from
           the language-term metas — order already resolved at import.
           The theme-option copy may drift in whitespace from the
           registered copy → normalized compare; several near-identical
           registrations exist → highest shared-line score wins. */
        let norm = |x: &str| x.split_whitespace().collect::<String>();
        let rn = norm(raw);
        let shared = |x: &str| {
            let xl: std::collections::BTreeSet<&str> =
                raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            x.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && xl.contains(l))
                .count()
        };
        /* Stale re-registrations pair a source with a same-language
           copy — a sibling that still equals raw is no translation. */
        pll_map
            .iter()
            .filter(|(p, t)| norm(p) == rn && norm(t) != rn)
            .map(|(_, t)| t.clone())
            .next()
            .or_else(|| {
                pll_map
                    .iter()
                    .filter(|(_, t)| norm(t) != rn)
                    .max_by_key(|(p, _)| shared(p))
                    .filter(|(p, _)| shared(p) > 3)
                    .map(|(_, t)| t.clone())
            })
            .unwrap_or_else(|| raw.to_string())
    };
    let bar = |key: &str| -> String {
        // Sites carry topbar HTML under either the literal `topbar_*_html`
        // option (aucoeur) or the theme-general `top-bar-*-text` field
        // (imagetravel) — literal key wins.
        let theme_key = if key == "topbar_left_html" {
            "top-bar-left-text"
        } else {
            "top-bar-right-text"
        };
        let raw = translated(&get(key).if_empty_else(|| tget(theme_key)));
        if raw.is_empty() {
            return String::new();
        }
        let expanded = crate::render::shortcode::expand_shortcodes(&raw, site_url, Some(conn));
        crate::url_util::rewrite_own_origin(&expanded, site_url, &menu_aliases(conn, site_url))
    };
    // Social profile URLs for JSON-LD sameAs — theme social options +
    // any outbound links in the top-bar payload.
    let mut same_as: Vec<String> = Vec::new();
    for k in [
        "top-bar-social-facebook",
        "top-bar-social-instagram",
        "top-bar-social-youtube",
        "top-bar-social-linkedin",
        "top-bar-social-twitter",
        "top-bar-social-tiktok",
    ] {
        let v = tget(k);
        if v.starts_with("http") {
            same_as.push(v);
        }
    }
    let name = get("blogname").if_empty_else(|| get("site_name"));
    let desc = get("blogdescription");
    // Sticky buttons from the ssb plugin payload: order string then btn map.
    let mut sticky: Vec<Value> = Vec::new();
    if let Ok(serde_json::Value::Object(o)) =
        crate::php_serialize::unserialize(get("ssb_buttons").as_bytes())
    {
        let mut btns: Vec<(i64, &serde_json::Value)> = o
            .get("btns")
            .and_then(|b| b.as_object())
            .map(|m| m.iter().filter_map(|(k, v)| k.parse::<i64>().ok().map(|n| (n, v))).collect())
            .unwrap_or_default();
        btns.sort_by_key(|(n, _)| *n);
        for (_, b) in btns {
            let text = b.get("btn_text").and_then(|v| v.as_str()).unwrap_or("");
            let link = b.get("btn_link").and_then(|v| v.as_str()).unwrap_or("");
            let icon = b.get("btn_icon").and_then(|v| v.as_str()).unwrap_or("").replace("fas fa-", "fa fa-");
            if !text.is_empty() {
                sticky.push(json!({
                    "text": text,
                    "url": crate::url_util::strip_origin(link, site_url, &menu_aliases(conn, site_url)),
                    "icon": icon,
                }));
            }
        }
    }
    json!({
        "name": name,
        "description": desc,
        "email": tget("top-bar-social-email"),
        "same_as": same_as,
        "logo": logo_img("logo", "traveltour-orig-logo"),
        "logo_fixed": logo_img("fixed-navigation-bar-logo", "traveltour-fixed-nav-logo"),
        "logo_mobile": logo_img("mobile-logo", "traveltour-orig-logo"),
        "topbar_left_html": bar("topbar_left_html"),
        "topbar_right_html": bar("topbar_right_html"),
        "copyright_left_html": crate::url_util::rewrite_own_origin(
            &crate::render::shortcode::expand_shortcodes(&get("copyright_left_html"), site_url, Some(conn)),
            site_url, &menu_aliases(conn, site_url)),
        "copyright_right_html": crate::url_util::rewrite_own_origin(
            &crate::render::shortcode::expand_shortcodes(&get("copyright_right_html"), site_url, Some(conn)),
            site_url, &menu_aliases(conn, site_url)),
        "sticky": sticky,
        "contact_slugs": serde_json::from_str::<serde_json::Value>(&get("contact_slugs")).unwrap_or(Value::Null),
        "enable_top_bar": tget("enable-top-bar") != "disable",
        "theme_css": theme_css,
        "tourmaster_css": tourmaster_css,
        "tourmaster_css_global": tourmaster_css_global,
        "tourmaster_css_room": tourmaster_css_room,
        "body_suffix": "wp-theme-traveltour wp-child-theme-traveltour-child gdlr-core-body tourmaster-body metaslider-plugin traveltour-body traveltour-body-front traveltour-full  traveltour-with-sticky-navigation gdlr-core-link-to-lightbox",
        "favicons": favicons_html(conn, site_url),
        "sticky_html": sticky_html(&sticky, &snippets_footer(&get, site_url), lang),
        /* The snippet rail ships its own sticky-slider-main-lang block —
           emitting the derived flag strip too would duplicate the flags. */
        "lang_flags_sticky": if get("snippets_footer_html").contains("sticky-slider-main-lang") {
            String::new()
        } else {
            lang_flags_sticky_html(conn, site_url)
        },
        "chaty_js": chaty_js(conn, site_url),
        "chaty_css": chaty_css(conn, site_url),
    })
}

trait IfEmpty { fn if_empty_else(self, f: impl FnOnce() -> String) -> String; }
impl IfEmpty for String {
    fn if_empty_else(self, f: impl FnOnce() -> String) -> String {
        if self.is_empty() { f() } else { self }
    }
}

/// `site_icon` media sizes → favicon links (WP emits 32/192/180/270px
/// variants when the attachment metadata carries them).
fn favicons_html(conn: &Connection, site_url: &str) -> String {
    let id: i64 = conn
        .query_row("SELECT CAST(value AS INTEGER) FROM settings WHERE key='site_icon'", [], |r| r.get(0))
        .unwrap_or(0);
    if id == 0 {
        return String::new();
    }
    let (path, sizes): (String, String) = match conn.query_row(
        "SELECT path, COALESCE(sizes_json,'{}') FROM media WHERE wp_id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ) {
        Ok(v) => v,
        _ => return String::new(),
    };
    let base = path.rsplitn(2, '.').nth(1).unwrap_or(&path);
    let ext = path.rsplitn(2, '.').next().unwrap_or("png");
    let sized = |w: i64, h: i64| -> Option<String> {
        let sj: serde_json::Value = serde_json::from_str(&sizes).unwrap_or(Value::Null);
        sj.as_object()?.values().find_map(|s| {
            let sw = s.get("width").and_then(|x| x.as_i64()).unwrap_or(0);
            let sh = s.get("height").and_then(|x| x.as_i64()).unwrap_or(0);
            (sw == w && sh == h).then(|| s.get("file").and_then(|f| f.as_str()).map(str::to_string))?
        })
        .or(Some(format!("{base}-{w}x{h}.{ext}")))
    };
    let mut out = String::new();
    if let Some(f) = sized(32, 32) {
        out.push_str(&format!("<link rel=\"icon\" href=\"{site_url}/wp-content/uploads/{}\" sizes=\"32x32\" />\n", dir_join(&path, &f)));
    }
    if let Some(f) = sized(192, 192) {
        out.push_str(&format!("<link rel=\"icon\" href=\"{site_url}/wp-content/uploads/{}\" sizes=\"192x192\" />\n", dir_join(&path, &f)));
    }
    if let Some(f) = sized(180, 180) {
        out.push_str(&format!("<link rel=\"apple-touch-icon\" href=\"{site_url}/wp-content/uploads/{}\" />\n", dir_join(&path, &f)));
    }
    if let Some(f) = sized(270, 270) {
        out.push_str(&format!("<meta name=\"msapplication-TileImage\" content=\"{site_url}/wp-content/uploads/{}\" />", dir_join(&path, &f)));
    }
    out
}
fn dir_join(path: &str, file: &str) -> String {
    match path.rfind('/') {
        Some(i) => format!("{}/{}", &path[..i], file),
        None => file.to_string(),
    }
}

/// Enabled Code Snippets' literal footer HTML (import captured it as
/// `snippets_footer_html` with `{{ site_url }}` tokens) — the custom
/// floating-button rail + lang strip the theme itself never stored.
/// Empty when the site has no footer snippet.
fn snippets_footer(get: &dyn Fn(&str) -> String, site_url: &str) -> String {
    get("snippets_footer_html").replace("{{ site_url }}", site_url)
}

/// Sticky rail: an imported snippet's own markup wins verbatim (it carries
/// the per-lang `sticky-{fr,es}` classes + its own lang strip); else the
/// ssb plugin payload → `.sticky-slider-main{ -one/-two }` wrappers.
fn sticky_html(btns: &[Value], snippet: &str, _lang: &str) -> String {
    if !snippet.is_empty() {
        return snippet.to_string();
    }
    if btns.is_empty() {
        return String::new();
    }
    let mut out = String::from("<div class=\"sticky-slider-main\">");
    for (i, b) in btns.iter().enumerate() {
        let text = b.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let url = b.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let icon = b.get("icon").and_then(|v| v.as_str()).unwrap_or("");
        out.push_str(&format!(
            "<div class=\"sticky-slider-main-{}\"><a rel=\"nofollow\" href=\"{}\"><span class=\"{}\"></span> {}</a></div>",
            ["one", "two", "three", "four"].get(i).copied().unwrap_or("one"),
            url, icon, text
        ));
    }
    out.push_str("</div>");
    out
}

/// Cross-language flag strip in the sticky rail — all configured langs,
/// flag PNG from media (falls back to the polylang .svg filename base).
/// Resolve a language's flag image to a URL that exists under the site's
/// own www/ docroot (or uploaded media) — "" when nothing exists, so the
/// template falls back to a text chip instead of a broken <img>.
/// Candidate order: uploaded 'flag {lang}' media → polylang flags/{lang}.png
/// → polylang vendor svg.
pub(crate) fn flag_src(conn: &Connection, site_url: &str, lang: &str) -> String {
    let site = site_url.trim_end_matches('/');
    let img: String = conn
        .query_row(
            "SELECT path FROM media WHERE LOWER(alt)=LOWER(?1) OR LOWER(title)=LOWER(?1) LIMIT 1",
            [format!("flag {lang}")],
            |r| r.get(0),
        )
        .unwrap_or_default();
    if !img.is_empty() {
        return format!("{site}/wp-content/uploads/{img}");
    }
    let www = conn
        .path()
        .map(|p| std::path::Path::new(p).parent().unwrap_or(std::path::Path::new("")).join("www"))
        .unwrap_or_default();
    let rels = [
        format!("wp-content/plugins/polylang/flags/{lang}.png"),
        format!("wp-content/plugins/polylang/vendor/wpsyntex/flags/{}", crate::lang::flag_file(lang)),
    ];
    for rel in rels {
        if www.join(&rel).is_file() {
            return format!("{site}/{rel}");
        }
    }
    String::new()
}

fn lang_flags_sticky_html(conn: &Connection, site_url: &str) -> String {
    let cfg = crate::lang::config(conn);
    if cfg.langs.len() < 2 {
        return String::new(); // single-language sites have no switcher
    }
    let mut out = String::new();
    for l in &cfg.langs {
        let url = format!("{}{}/", site_url.trim_end_matches('/'), crate::lang::url_prefix(&cfg, l));
        let src = flag_src(conn, site_url, l);
        let inner = if src.is_empty() {
            l.to_uppercase()
        } else {
            format!("<img src=\"{src}\"/>")
        };
        out.push_str(&format!(
            "<a class=\"lang-flag mobile {l}\" href=\"{url}\"><span style=\"color: #fff; margin-left: 12px\">{inner}</span></a>\n"
        ));
    }
    out
}

/// Chaty widget: `cht_*`/`chaty_*` options → the plugin's inline
/// `chaty_settings` object + script tags. Absent options → empty string
/// (no chat markup on sites without the plugin).
fn chaty_js(conn: &Connection, site_url: &str) -> String {
    let get = |k: &str| -> String {
        conn.query_row("SELECT value FROM settings WHERE key=?1", [k], |r| r.get::<_, String>(0))
            .unwrap_or_default()
    };
    let total = get("chaty_total_settings").parse::<i64>().unwrap_or(0);
    if total == 0 {
        return String::new();
    }
    // Channel list: cht_social_{n}_{Name} serialized rows.
    let mut channels = Vec::new();
    for n in 0..total.max(1) {
        let mut st = match conn.prepare(
            "SELECT key, value FROM settings WHERE key LIKE ?1",
        ) {
            Ok(s) => s,
            _ => return String::new(),
        };
        let pat = format!("cht_social_{n}_%");
        let rows: Vec<(String, String)> = st
            .query_map(rusqlite::params![pat], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map(|it| it.flatten().collect())
            .unwrap_or_default();
        if !rows.is_empty() {
            for (k, v) in rows {
                let name = k.rsplit('_').next().unwrap_or("");
                if let Ok(v) = crate::php_serialize::unserialize(v.as_bytes()) {
                    channels.push(json!({"id": name, "value": v.get("value").cloned().unwrap_or(Value::Null)}));
                }
            }
        }
    }
    let widget = json!({
        "id": 0,
        "identifier": 0,
        "settings": {
            "cta_type": "simple-view",
            "default_state": get("chaty_default_state").if_empty("click".into()),
            "position": get("cht_position").if_empty("right".into()),
            "cta_text": get("cht_cta"),
            "cta_bg_color": get("cht_cta_bg_color"),
            "icon_view": get("chaty_icons_view").if_empty("vertical".into()),
            "bottom_spacing": get("cht_bottom_spacing").if_empty("25".into()),
            "side_spacing": get("cht_side_spacing").if_empty("25".into()),
        },
        "channels": channels,
    });
    let settings = json!({
        "ajax_url": format!("{site_url}/wp-admin/admin-ajax.php"),
        "analytics": "0",
        "capture_analytics": "0",
        "token": get("cht_token"),
        "chaty_widgets": [widget],
    });
    format!(
        "<script id=\"chaty-front-end-js-extra\">\nvar chaty_settings = {};\n</script>\n\
         <script defer id=\"chaty-front-end-js\" src=\"{site_url}/wp-content/plugins/chaty/dist/js/script.js\"></script>\n\
         <script defer id=\"chaty-mail-check-js\" src=\"{site_url}/wp-content/plugins/chaty/admin/assets/js/mailcheck.js\"></script>\n\
         <script id=\"chaty-picmo-js-js\" src=\"{site_url}/wp-content/plugins/chaty/admin/assets/js/picmo-umd.min.js\"></script>",
        settings
    )
}
fn chaty_css(conn: &Connection, site_url: &str) -> String {
    let has: bool = conn
        .query_row("SELECT 1 FROM settings WHERE key='chaty_total_settings' LIMIT 1", [], |r| {
            r.get::<_, i64>(0)
        })
        .is_ok();
    if has {
        format!("<link rel='stylesheet' id='chaty-front-css-css' href='{site_url}/wp-content/plugins/chaty/dist/css/style.css' media='all' />")
    } else {
        String::new()
    }
}

trait IfEmpty2 { fn if_empty(self, f: String) -> String; }
impl IfEmpty2 for String {
    fn if_empty(self, f: String) -> String { if self.is_empty() { f } else { self } }
}
