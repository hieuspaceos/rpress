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
        let mut istmt = conn.prepare(
            "SELECT m.id, m.wp_term_id, m.name, m.lang, i.id, i.parent_id, i.ord, \
             i.label, COALESCE(i.url,'') FROM menus m \
             JOIN menu_items i ON i.menu_id=m.id ORDER BY i.ord",
        )?;
        let rows = istmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, String>(8)?,
            ))
        })?;
        let mut by_menu: HashMap<i64, (i64, String, Option<String>, Vec<Value>)> = HashMap::new();
        for row in rows {
            let (mid, wpid, name, mlang, iid, parent, ord, label, iurl) = row?;
            let entry = by_menu
                .entry(mid)
                .or_insert_with(|| (wpid, name, mlang, Vec::new()));
            entry.3.push(json!({
                "id": iid, "parent_id": parent.unwrap_or(0), "ord": ord,
                "title": label.unwrap_or_default(), "url": iurl, "children": [],
            }));
        }
        for (mid, (wpid, name, _mlang, flat)) in by_menu {
            let tree = build_menu_tree(flat);
            let v = json!(tree);
            menus.insert(name.clone(), v.clone());
            menus.insert(mid.to_string(), v.clone());
            menus.insert(format!("wp:{wpid}"), v);
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
        "tour_base": tour_base,
        "tour_base_url": format!("{site_url}{prefix}/{tour_base}/"),
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
        let child_ids: Vec<i64> = nodes
            .iter()
            .filter(|(_, n)| n.get("parent_id").and_then(|p| p.as_i64()) == Some(id))
            .map(|(cid, _)| *cid)
            .collect();
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
        let is_root = nodes
            .get(&id)
            .and_then(|n| n.get("parent_id").and_then(|p| p.as_i64()))
            .map(|p| p == 0 || !nodes.contains_key(&p))
            .unwrap_or(true);
        if is_root {
            if let Some(n) = attach(id, &mut nodes, &mut seen) {
                roots.push(n);
            }
        }
    }
    roots
}

