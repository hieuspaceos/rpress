//! Menu data layer — `nav-menus.php` equivalent (`menus` + `menu_items`).
//! Replaces the flat `misc::list_menus`/`save_menu_items` helpers; every write
//! that touches more than one row runs inside a transaction.
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct Menu {
    pub id: i64,
    pub name: String,
    pub lang: String,
    /// Imported but has no consumer today (Manage Locations writes the
    /// `theme_locations` table instead; kept for back-compat).
    #[allow(dead_code)]
    pub location: String,
    /// WP "Automatically add new top-level pages to this menu" checkbox.
    pub auto_add: bool,
}

/// One `menu_items` row plus the joined source object (post or term) so the
/// editor can render the WP "Original:" link and the item type label.
pub struct MenuItem {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub ord: i64,
    pub label: String,
    pub url: String,
    #[allow(dead_code)] // reserved: raw source object id (join already resolved)
    pub object_id: Option<i64>,
    pub item_type: String,
    // post join (post_type items): id, title, slug, lang, kind
    pub post_id: Option<i64>,
    pub post_title: String,
    pub post_slug: String,
    pub post_lang: String,
    pub post_kind: String,
    // term join (taxonomy items): id, name, slug, taxonomy
    pub term_id: Option<i64>,
    pub term_name: String,
    #[allow(dead_code)] // slug unused — Original links to the admin term editor
    pub term_slug: String,
    pub term_taxonomy: String,
    // nav-menus.php "Show advanced menu properties" fields
    pub title_attr: String,
    pub target: String,
    pub classes: String,
    pub xfn: String,
    pub description: String,
}

/// Per-item fields posted by the Menu Structure editor (`*-{id}` keys).
pub struct ItemForm {
    pub id: i64,
    pub ord: i64,
    pub parent_id: Option<i64>,
    pub label: String,
    pub url: String,
    pub title_attr: String,
    pub target: String,
    pub classes: String,
    pub xfn: String,
    pub description: String,
    pub delete: bool,
}

/// A new item appended by the Add-to-Menu metaboxes.
pub struct NewItem {
    pub label: String,
    pub url: String,
    pub object_id: Option<i64>,
    pub item_type: String,
}

pub fn list_menus(db: &Path) -> Result<Vec<Menu>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("menus__list"))?;
    let rows = st
        .query_map([], |r| {
            Ok(Menu { id: r.get(0)?, name: r.get(1)?, lang: r.get(2)?, location: r.get(3)?, auto_add: r.get(4)? })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn get_menu(db: &Path, id: i64) -> Result<Option<Menu>> {
    let c = crate::db::conn(db)?;
    Ok(c
        .query_row(sql!("menus__get"), [id], |r| {
            Ok(Menu { id: r.get(0)?, name: r.get(1)?, lang: r.get(2)?, location: r.get(3)?, auto_add: r.get(4)? })
        })
        .optional()?)
}


/// Flat list ordered by `ord` (items were always stored in depth-first ord,
/// so this is the WP editor order). Depth is derived from `parent_id`.
pub fn list_items(db: &Path, menu_id: i64) -> Result<Vec<MenuItem>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("menus__items"))?;
    let rows = st
        .query_map([menu_id], |r| {
            Ok(MenuItem {
                id: r.get(0)?,
                parent_id: r.get(1)?,
                ord: r.get(2)?,
                label: r.get(3)?,
                url: r.get(4)?,
                object_id: r.get(5)?,
                item_type: r.get(6)?,
                post_id: r.get(7)?,
                post_title: r.get(8)?,
                post_slug: r.get(9)?,
                post_lang: r.get(10)?,
                post_kind: r.get(11)?,
                term_id: r.get(12)?,
                term_name: r.get(13)?,
                term_slug: r.get(14)?,
                term_taxonomy: r.get(15)?,
                title_attr: r.get(16)?,
                target: r.get(17)?,
                classes: r.get(18)?,
                xfn: r.get(19)?,
                description: r.get(20)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn create_menu(db: &Path, name: &str, lang: &str) -> Result<i64> {
    let c = crate::db::conn(db)?;
    c.execute(sql!("menus__create"), params![name.trim(), lang])?;
    Ok(c.last_insert_rowid())
}

/// Items first, then the menu row — one transaction.
pub fn delete_menu(db: &Path, id: i64) -> Result<()> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    tx.execute(sql!("menus__delete_items"), [id])?;
    tx.execute(sql!("menus__delete_menu"), [id])?;
    tx.commit()?;
    Ok(())
}

/// Save Menu: rename + auto-add flag + replace-tree (update ord/parent/label/
/// url/advanced fields or delete each submitted item) in ONE transaction.
/// `auto_add` is `Some` only when the menu edit form was posted (the checkbox
/// lives on that form; other save callers leave the column untouched).
pub fn save_tree(db: &Path, menu_id: i64, name: Option<&str>, items: &[ItemForm], auto_add: Option<bool>) -> Result<()> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    if let Some(n) = name {
        if !n.trim().is_empty() {
            tx.execute(sql!("menus__rename"), params![n.trim(), menu_id])?;
        }
    }
    if let Some(a) = auto_add {
        tx.execute(sql!("menus__set_auto_add"), params![a as i64, menu_id])?;
    }
    for it in items {
        if it.delete {
            tx.execute(sql!("menus__delete_item"), params![it.id, menu_id])?;
        } else {
            tx.execute(
                sql!("menus__update_item"),
                params![
                    it.ord, it.parent_id, it.label, it.url,
                    it.title_attr, it.target, it.classes, it.xfn, it.description,
                    it.id, menu_id
                ],
            )?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Append items as last top-level items, in the order given (check order).
pub fn add_items(db: &Path, menu_id: i64, items: &[NewItem]) -> Result<()> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    let mut ord: i64 = tx.query_row(sql!("menus__max_ord"), [menu_id], |r| r.get(0))?;
    for it in items {
        ord += 1;
        tx.execute(
            sql!("menus__insert_item"),
            params![menu_id, ord, it.label, it.url, it.object_id, it.item_type],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Reposition one item (no-JS fallback for the "Up one/Down one/Under …/Out
/// from under …/To the top" move links). Works on the flat ord-ordered list;
/// moves that change parent keep DFS order, subtree moves carry descendants.
/// After the reorder, `ord` is renumbered 1..n so the stored order stays
/// depth-first-contiguous. Returns false when the move was a no-op.
pub fn move_item(db: &Path, menu_id: i64, id: i64, dir: &str) -> Result<bool> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    let flat = {
        let mut st = tx.prepare(sql!("menus__items"))?;
        let rows = st
            .query_map([menu_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?)))?
            .collect::<rusqlite::Result<Vec<(i64, Option<i64>)>>>()?;
        rows
    };
    let Some(idx) = flat.iter().position(|(iid, _)| *iid == id) else {
        return Ok(false);
    };
    let parent_of = |iid: i64| flat.iter().find(|(x, _)| *x == iid).and_then(|(_, p)| *p);
    let is_desc = |mut cand: i64, anc: i64| -> bool {
        let mut hops = 0;
        while let Some(p) = parent_of(cand) {
            if p == anc {
                return true;
            }
            cand = p;
            hops += 1;
            if hops > 64 {
                return false;
            }
        }
        false
    };
    // Index one past the subtree rooted at `i` (descendants immediately
    // follow their ancestor in DFS ord).
    let subtree_end = |ord: &[i64], i: usize| -> usize {
        let root = ord[i];
        let mut e = i + 1;
        while e < ord.len() && is_desc(ord[e], root) {
            e += 1;
        }
        e
    };
    let mut order: Vec<i64> = flat.iter().map(|(iid, _)| *iid).collect();
    let mut new_parent = flat[idx].1;
    match dir {
        // Move the whole subtree before the contiguous subtree block that
        // ends right above it (WP nav-menus "Up one" swaps siblings; a bare
        // row swap would strand the previous parent's descendants — the
        // audit's orphan bug). No-op when the item's parent sits above it.
        "up" if idx > 0 => {
            let end = subtree_end(&order, idx);
            // Previous row's subtree swallowing the moving item → it is an
            // ancestor; moving up would break DFS order, so no-op.
            if subtree_end(&order, idx - 1) > idx {
                return Ok(false);
            }
            // Largest contiguous subtree ending exactly at idx: climb
            // order[idx-1]'s ancestor chain while each ancestor's subtree
            // ends at idx (a trailing chain of last-children rolls up).
            let mut start = idx - 1;
            while let Some(par) = parent_of(order[start]) {
                match order.iter().position(|x| *x == par) {
                    Some(ppos) if subtree_end(&order, ppos) == idx => start = ppos,
                    _ => break,
                }
            }
            let block: Vec<i64> = order.drain(idx..end).collect();
            order.splice(start..start, block);
        }
        // Same shape mirrored: the subtree moves past the contiguous block
        // that starts right after it. No-op when nothing follows.
        "down" if idx + 1 < order.len() => {
            let end = subtree_end(&order, idx);
            if end >= order.len() {
                return Ok(false);
            }
            // Last child of a parent can't slide past the next ancestor-
            // level subtree — that would strand it outside its parent
            // (the `out` move handles that reparent).
            if let Some(p) = flat[idx].1 {
                if !is_desc(order[end], p) {
                    return Ok(false);
                }
            }
            let next_end = subtree_end(&order, end);
            let block: Vec<i64> = order.drain(idx..end).collect();
            order.splice(next_end - block.len()..next_end - block.len(), block);
        }
        "top" if idx > 0 => {
            let end = subtree_end(&order, idx);
            let block: Vec<i64> = order.drain(idx..end).collect();
            order.splice(0..0, block);
            new_parent = None;
        }
        // Child of the item directly above; position is already correct —
        // only the parent changes.
        "under" if idx > 0 => new_parent = Some(flat[idx - 1].0),
        // Sibling of the parent, placed right after the parent's subtree.
        "out" => {
            let Some(par) = flat[idx].1 else { return Ok(false) };
            new_parent = flat
                .iter()
                .find(|(iid, _)| *iid == par)
                .and_then(|(_, p)| *p);
            let end = subtree_end(&order, idx);
            let block: Vec<i64> = order.drain(idx..end).collect();
            if let Some(ppos) = order.iter().position(|x| *x == par) {
                let p_end = subtree_end(&order, ppos);
                order.splice(p_end..p_end, block);
            } else {
                // Orphaned parent id: promote to top level, keep position.
                let back = order.len().min(idx);
                order.splice(back..back, block);
            }
        }
        _ => return Ok(false),
    }
    apply_order(&tx, &order, &[(id, new_parent)], &flat, menu_id)?;
    tx.commit()?;
    Ok(true)
}

/// Persist the renumbered order plus parent overrides inside the open tx.
/// `parents` overrides `flat`'s stored parent for the listed ids.
fn apply_order(
    tx: &Connection,
    order: &[i64],
    parents: &[(i64, Option<i64>)],
    flat: &[(i64, Option<i64>)],
    menu_id: i64,
) -> Result<()> {
    for (pos, iid) in order.iter().enumerate() {
        let par = match parents.iter().find(|(pid, _)| pid == iid) {
            Some((_, p)) => *p,
            None => flat.iter().find(|(x, _)| x == iid).and_then(|(_, p)| *p),
        };
        tx.execute(
            sql!("menus__move_item"),
            params![pos as i64 + 1, par, *iid, menu_id],
        )?;
    }
    Ok(())
}

/// Published posts of a kind for an add-box panel; `lang` "" = no filter.
pub fn panel_posts(db: &Path, kind: &str, lang: &str, recent_only: bool) -> Result<Vec<(i64, String)>> {
    let c = crate::db::conn(db)?;
    let sql = if recent_only {
        sql!("menus__recent_posts")
    } else {
        sql!("menus__all_posts")
    };
    let mut st = c.prepare(sql)?;
    let rows = st
        .query_map(params![kind, lang], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Title + kind for a post being appended (label default + type label).
pub fn post_label(db: &Path, id: i64) -> Result<Option<(String, String)>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("menus__post_label"), [id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?)
}

/// Name + taxonomy for a term being appended.
pub fn term_label(db: &Path, id: i64) -> Result<Option<(String, String)>> {
    let c = crate::db::conn(db)?;
    Ok(c.query_row(sql!("menus__term_label"), [id], |r| Ok((r.get(0)?, r.get(1)?))).optional()?)
}

/// Theme locations the active theme declares — `settings.theme_locations_available`
/// JSON array (`["primary","footer"]` default when unset/corrupt). The theme
/// (template) owns the list; admin only assigns menus to it.
pub fn available_locations(db: &Path) -> Vec<String> {
    crate::db::comments::setting_get(db, "theme_locations_available")
        .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec!["primary".into(), "footer".into()])
}

/// Manage Locations assignments: `location → menu_id` for declared locations.
/// Stored rows for undeclared locations are ignored by the UI.
pub fn list_locations(db: &Path, locations: &[String]) -> Result<Vec<(String, String, Option<i64>)>> {
    let c = crate::db::conn(db)?;
    let mut st = c.prepare(sql!("menus__locations_list"))?;
    let cur: std::collections::HashMap<(String, String), Option<i64>> = st
        .query_map([], |r| {
            Ok((
                (r.get::<_, String>(0)?, r.get::<_, String>(1)?),
                r.get::<_, Option<i64>>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<std::collections::HashMap<(String, String), Option<i64>>>>()?;
    /* Rows exist only for assigned pairs; declared locations with no
       row appear once per site lang as unassigned — the caller supplies
       the (location, lang) grid. */
    Ok(locations
        .iter()
        .map(|loc| {
            cur.iter()
                .find(|((l, _), _)| l == loc)
                .map(|((_, lang), mid)| (loc.clone(), lang.clone(), *mid))
                .unwrap_or_else(|| (loc.clone(), String::new(), None))
        })
        .collect())
}

/// Menu assigned to a theme location for `lang`; `None` = unassigned.
pub fn location_menu(db: &Path, location: &str, lang: &str) -> Option<i64> {
    let c = crate::db::conn(db).ok()?;
    c.query_row(
        "SELECT menu_id FROM theme_locations WHERE location=?1 AND lang=?2",
        rusqlite::params![location, lang],
        |r| r.get::<_, Option<i64>>(0),
    )
    .ok()
    .flatten()
}

/// Persist the Manage Locations table: (location, lang, menu_id) rows —
/// `menu_id` NULL deletes the binding. One transaction.
pub fn save_locations(db: &Path, assignments: &[(String, String, Option<i64>)]) -> Result<()> {
    let mut c = crate::db::conn(db)?;
    let tx = c.transaction()?;
    for (loc, lang, menu_id) in assignments {
        match menu_id {
            Some(_) => {
                tx.execute(sql!("menus__location_set"), params![loc, lang, menu_id])?;
            }
            None => {
                tx.execute(
                    "DELETE FROM theme_locations WHERE location=?1 AND lang=?2",
                    params![loc, lang],
                )?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

/// WP "Automatically add new top-level pages": after a *new* top-level page is
/// saved, append it as the last item of every menu with `auto_add=1` that
/// doesn't already link it. No-op for existing pages, child pages, or other
/// kinds — caller checks `f.id.is_none()` + `f.kind == "page"`.
pub fn auto_add_page(db: &Path, post_id: i64) -> Result<()> {
    let c = crate::db::conn(db)?;
    let top: i64 = c.query_row(sql!("menus__page_toplevel"), [post_id], |r| r.get(0))?;
    if top == 0 {
        return Ok(());
    }
    let Some((title, _kind)) = post_label(db, post_id)? else {
        return Ok(());
    };
    let menu_ids: Vec<i64> = {
        let mut st = c.prepare(sql!("menus__auto_add_menus"))?;
        let rows = st.query_map([], |r| r.get(0))?;
        rows.collect::<rusqlite::Result<Vec<i64>>>()?
    };
    for menu_id in menu_ids {
        let has: i64 = c.query_row(sql!("menus__auto_add_exists"), params![menu_id, post_id], |r| r.get(0))?;
        if has > 0 {
            continue;
        }
        add_items(
            db,
            menu_id,
            &[NewItem {
                label: title.clone(),
                url: String::new(),
                object_id: Some(post_id),
                item_type: "post_type".into(),
            }],
        )?;
    }
    Ok(())
}

/// Per-user `nav-menus.php` Screen Options, JSON in `settings` under
/// `menus_screen_{user_id}`: `fields` = advanced item fields shown (WP's
/// `managenav-menuscolumnshidden` complement), `hidden` = collapsed/removed
/// accordion metabox ids (`add-{key}` keys like "page", "category", "tax-x").
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct MenuScreen {
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub hidden: Vec<String>,
}

impl MenuScreen {
    pub fn load(db: &Path, user_id: i64) -> Self {
        crate::db::comments::setting_get(db, &format!("menus_screen_{user_id}"))
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }
    pub fn save(db: &Path, user_id: i64, o: &Self) -> Result<()> {
        crate::db::comments::setting_set(db, &format!("menus_screen_{user_id}"), &serde_json::to_string(o)?)
    }
    /// True when the advanced field is enabled in Screen Options.
    pub fn field(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f == name)
    }
    /// True when the accordion metabox `add-{key}` is enabled.
    pub fn section(&self, key: &str) -> bool {
        !self.hidden.iter().any(|h| h == key)
    }
}

/// Batch post lookup for menu resolution: wp_id → (id, slug, lang, kind,
/// title) for every published post whose wp_id is in `ids`. Chunked IN
/// queries (400/bind-limit) — the SQL is generated, so it lives here in
/// db/, not in the renderer.
pub(crate) fn posts_by_wp_ids(
    conn: &rusqlite::Connection,
    ids: &[i64],
) -> std::collections::HashMap<i64, (i64, String, String, String, String)> {
    let mut out = std::collections::HashMap::new();
    for chunk in ids.chunks(400) {
        if chunk.is_empty() { continue; }
        let qs = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!(
            "SELECT wp_id, id, slug, lang, kind, title FROM posts WHERE status='publish' AND wp_id IN ({qs})"
        );
        if let Ok(mut st) = conn.prepare(&q) {
            if let Ok(rows) = st.query_map(rusqlite::params_from_iter(chunk.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, String>(4)?, r.get::<_, String>(5)?))
            }) {
                for row in rows.flatten() {
                    out.insert(row.0, (row.1, row.2, row.3, row.4, row.5));
                }
            }
        }
    }
    out
}

/// Batch term lookup for menu resolution: terms.id → (taxonomy, slug).
pub(crate) fn terms_by_ids(
    conn: &rusqlite::Connection,
    ids: &[i64],
) -> std::collections::HashMap<i64, (String, String)> {
    let mut out = std::collections::HashMap::new();
    for chunk in ids.chunks(400) {
        if chunk.is_empty() { continue; }
        let qs = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!("SELECT id, taxonomy, slug FROM terms WHERE id IN ({qs})");
        if let Ok(mut st) = conn.prepare(&q) {
            if let Ok(rows) = st.query_map(rusqlite::params_from_iter(chunk.iter()), |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
            }) {
                for row in rows.flatten() {
                    out.insert(row.0, (row.1, row.2));
                }
            }
        }
    }
    out
}
