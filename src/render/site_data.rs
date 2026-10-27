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
    // tours (kind='tour' joined to tours table) with term slugs per taxonomy.
    // '-copy' slugs are WP duplicated drafts published post-capture — the
    // reference predates them, so exclude to match live selection.
    let mut tours = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id, \
             t.duration_days, COALESCE(t.price_range,''), COALESCE(t.rating,0), t.max_people, \
             COALESCE(p.published_at,'') \
             FROM posts p JOIN tours t ON t.post_id=p.id \
             WHERE p.kind='tour' AND p.lang=?1 AND p.status='publish' \
             AND p.slug NOT LIKE '%-copy' \
             ORDER BY p.menu_order, p.published_at",
        )?;
        let rows = stmt.query_map([lang], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, f64>(7)?,
                r.get::<_, Option<i64>>(8)?,
                r.get::<_, String>(9)?,
            ))
        })?;
        for row in rows {
            let (id, slug, title, excerpt, fid, days, price, rating, max_people, pub_at) = row?;
            let mut terms = Vec::new();
            let mut tstmt = conn.prepare(
                "SELECT taxonomy, slug, name FROM post_terms WHERE post_id=?1",
            )?;
            let trows = tstmt.query_map([id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
            })?;
            for t in trows.flatten() {
                terms.push(json!({"taxonomy": t.0, "slug": t.1, "name": t.2}));
            }
            // tour option meta: ribbon, duration text, departure, price, rating
            let opt = conn
                .query_row(
                    "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                     AND meta_key='tourmaster-tour-option' LIMIT 1",
                    [id],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .and_then(|raw| {
                    crate::php_serialize::unserialize(raw.as_bytes()).ok()
                });
            let opt_s = |k: &str| -> String {
                opt.as_ref()
                    .and_then(|v| v.get(k))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            // promo ribbon falls back to style term names
            let mut promo_text = opt_s("promo-text");
            if promo_text.is_empty() {
                promo_text = terms
                    .iter()
                    .filter(|t| {
                        t.get("taxonomy").and_then(|x| x.as_str())
                            == Some(if lang == "es" { "estilo" } else { "style" })
                    })
                    .filter_map(|t| t.get("name").and_then(|x| x.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ");
            }
            let promo_bg = opt_s("promo-text-ribbon-background");
            // rating + reviewer count live in tourmaster-tour-rating meta
            let mut review_count: i64 = 0;
            let mut rating = rating;
            if let Ok(raw) = conn.query_row::<String, _, _>(
                "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                 AND meta_key='tourmaster-tour-rating' LIMIT 1",
                [id],
                |r| r.get(0),
            ) {
                if let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) {
                    if let Some(sc) = v
                        .get("score")
                        .and_then(|x| x.as_f64().or_else(|| x.as_i64().map(|i| i as f64)))
                    {
                        rating = sc / 2.0;
                    }
                    review_count = v
                        .get("reviewer")
                        .and_then(|x| x.as_i64())
                        .unwrap_or(0);
                }
            }
            // display price: tour-price-text > tourmaster-tour-price > price_range
            let raw_price = {
                let t = opt_s("tour-price-text");
                if !t.is_empty() {
                    t
                } else {
                    conn.query_row::<String, _, _>(
                        "SELECT meta_value FROM post_meta WHERE post_id=?1 \
                         AND meta_key='tourmaster-tour-price' LIMIT 1",
                        [id],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| price.clone())
                }
            };
            // tour permalinks use display_slug (raw slug can be degenerate,
            // e.g. 'x' on 6416) — the value file path + canonical agree on.
            let tslug = crate::render_tour::display_slug(conn, id, &slug, &title);
            tours.push(json!({
                "id": id, "slug": slug, "title": title, "excerpt": excerpt,
                "url": url("tour", &tslug), "image": img(fid),
                "alt": fid.and_then(&media_row).map(|(_, a)| a.clone()),
                "duration_days": days, "price": price, "rating": rating,
                "max_people": max_people, "terms": terms,
                "duration_text": opt_s("duration-text"),
                "departure": opt_s("departure-location"),
                "promo_text": promo_text,
                "promo_color": opt_s("promo-text-ribbon-text-color"),
                "promo_bg": promo_bg,
                "promo_bg_alpha": hex_rgba(&promo_bg, 0.5),
                "price_display": fmt_tour_price(&raw_price),
                "review_count": review_count, "date": pub_at,
            }));
        }
    }

    // testimonials: site-review posts; reviewer name from _submitted meta
    let mut reviews = Vec::new();
    {
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
    }

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

    // tourmaster_general options: search page url + month filter list, both
    // consumed by elements/tour_search.html.
    let general: Option<Value> = crate::db::plugin_setting_raw_conn(&conn, "tourmaster", "tourmaster_general")
        .and_then(|raw| crate::php_serialize::unserialize(raw.as_bytes()).ok());
    let gen_opt = |key: &str| -> Option<String> {
        general
            .as_ref()
            .and_then(|g| g.get(key))
            .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            .filter(|s| !s.is_empty())
    };

    // tourmaster_get_template_url('search'): permalink of the 'search-page'
    // option resolved to the current language; home url when unset.
    let search_url = {
        let mut sp = gen_opt("search-page").and_then(|s| s.parse::<i64>().ok());
        if lang != lang_cfg.primary {
            if let Some(sid) = sp {
                // N-lang: resolve the configured page's member in this
                // language (post_translation_members); the legacy fr/es
                // pair table still resolves for sites imported before
                // members sync.
                let member: Option<i64> = conn
                    .query_row(
                        "SELECT m2.post_id FROM post_translation_members m1 \
                         JOIN post_translation_members m2 ON m2.group_id=m1.group_id AND m2.lang=?2 \
                         WHERE m1.post_id=?1 LIMIT 1",
                        rusqlite::params![sid, lang],
                        |r| r.get(0),
                    )
                    .ok();
                sp = member
                    .or_else(|| {
                        let pair: Option<(Option<i64>, Option<i64>)> = conn
                            .query_row(
                                sql!("render__select_fr_post_id_es_post_id"),
                                [sid],
                                |r| Ok((r.get(0)?, r.get(1)?)),
                            )
                            .ok();
                        pair.and_then(|(f, e)| if f == Some(sid) { e } else { None })
                    })
                    .or(sp);
            }
        }
        match sp.and_then(|sid| slug_of(conn, sid)) {
            Some((slug, kind)) => page_url(conn, site_url, lang, &kind, &slug, tour_base),
            None => format!("{site_url}{prefix}/"),
        }
    };

    // 'month' combobox: N months starting this month (PHP
    // search-month-amount + date_i18n('F Y')); days→civil arithmetic, no chrono.
    let month_names = crate::lang::month_names(lang);
    let month_amount: i64 = gen_opt("search-month-amount")
        .and_then(|s| s.parse().ok())
        .unwrap_or(12)
        .clamp(0, 36);
    let months: Vec<Value> = {
        let days = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / 86400)
            .unwrap_or(0)) as i64;
        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let (mut y, m) = (if mp >= 10 { y + 1 } else { y }, mp + 3 - (if mp >= 10 { 12 } else { 0 }));
        (0..month_amount)
            .map(|i| {
                let mi = m + i;
                y += (mi - 1) / 12;
                let mm = (mi - 1) % 12 + 1;
                json!({"slug": format!("{y:04}-{mm:02}"),
                       "name": format!("{} {}", month_names[(mm - 1) as usize], y)})
            })
            .collect()
    };

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

    Ok(JVal::from_serialize(json!({
        "posts": posts,
        "tours": tours,
        "reviews": reviews,
        "terms": terms,
        "menus": menus,
        "pages": pages,
        "lang": lang,
        "url_prefix": prefix,
        "site_url": site_url,
        "tour_base": tour_base,
        "tour_base_url": format!("{site_url}{prefix}/{tour_base}/"),
        "search_url": search_url,
        "search_months": months,
    })))
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

/// "#10873b" + alpha -> "rgba(16, 135, 59, 0.5)" for the ribbon corner.
fn hex_rgba(hex: &str, alpha: f64) -> String {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return String::new();
    }
    let byte = |i: usize| -> Option<u8> { u8::from_str_radix(&h[i..i + 2], 16).ok() };
    match (byte(0), byte(2), byte(4)) {
        (Some(r), Some(g), Some(b)) => format!("rgba({r}, {g}, {b}, {alpha})"),
        _ => String::new(),
    }
}

/// Format a tour price string: bare digits -> "1,825 USD", else raw text.
fn fmt_tour_price(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.chars().all(|c| c.is_ascii_digit()) {
        let mut out = String::new();
        for (i, c) in t.chars().enumerate() {
            if i > 0 && (t.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        format!("{out} USD")
    } else {
        t.to_string()
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

