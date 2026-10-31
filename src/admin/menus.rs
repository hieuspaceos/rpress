//! Appearance → Menus (`nav-menus.php` parity).
//!
//! Screen layout: tab strip (Edit Menus | Manage Locations), selector bar,
//! left accordion metaboxes (Pages/Posts/Tours/Custom Links/Categories/
//! taxonomies) + right Menu Structure column with expandable item blocks.
//! Move links are tiny POST forms (`form="menu-move"` + `formaction=…?dir=…`)
//! so every move works without JS; `menu-editor.js` intercepts them for
//! client-side reorder. Advanced item fields (Title Attribute, Link Target,
//! CSS Classes, XFN, Description) render under each item, visibility toggled
//! per-user by Screen Options (`settings.menus_screen_{user_id}`).
//!
//! Cuts vs core (recorded in phase file): widget-based editor.
use super::*;
use crate::admin::posts::public_path;

/// `GET /admin/menus?menu={id}` — `menu=0` = create view; absent/unknown id
/// falls back to the first menu; no menus at all = "create your first menu".
pub(crate) async fn menus_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // rusqlite is sync — offload conn+queries+render to a blocking thread
    // (same pattern as dashboard; audit P1).
    let (s2, sc2) = (s, sc.clone());
    return tokio::task::spawn_blocking(move || menus_page_sync(s2, &sc2, &q))
        .await
        .unwrap_or_else(|e| {
            eprintln!("menus_page join: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        });
}

fn menus_page_sync(s: Sess, sc: &SiteCtx, q: &HashMap<String, String>) -> Response { // gate-allow: blocking by contract — only called inside spawn_blocking
    let menus = crud::list_menus(&sc.db).unwrap_or_default();
    let want = q.get("menu").and_then(|v| v.parse::<i64>().ok());
    let creating = menus.is_empty() || matches!(want, Some(0));
    // Absent or unknown id → first menu (WP default selection).
    let menu = if creating {
        None
    } else {
        want.and_then(|id| crud::get_menu(&sc.db, id).unwrap_or(None))
            .or_else(|| menus.first().and_then(|m| crud::get_menu(&sc.db, m.id).unwrap_or(None)))
    };
    let action = q.get("action").map(|s| s.as_str()).unwrap_or("");
    let locations_view = action == "locations";
    let menus_screen = crud::MenuScreen::load(&sc.db, s.user_id);
    let msg = q.get("msg").map(|s| s.as_str()).unwrap_or("");
    let notice = match msg {
        "created" => "Your new menu has been successfully created.",
        "deleted" => "The menu has been successfully deleted.",
        "saved" => "The menu has been successfully updated.",
        "added" => "The menu item has been successfully updated.",
        "moved" => "The menu item has been successfully updated.",
        "locations" => "The menu locations have been updated.",
        "screen" => "Screen options have been updated.",
        _ => "",
    };
    let mut body = chrome("menus", &s);
    // Screen Options tab — edit view only (WP's Locations tab has no
    // toggleable elements).
    if !locations_view {
        body.push_str(&screen_options_html(sc, &s, &menus_screen));
    }
    body.push_str("<div class=\"wrap\"><h1 class=\"wp-heading-inline\">Menus</h1>");
    // Tab strip — Edit Menus | Manage Locations; `?menu=N` is preserved.
    let menu_q = menu.as_ref().map(|m| format!("menu={}&amp;", m.id)).unwrap_or_default();
    body.push_str(&format!(
        "<h2 class=\"nav-tab-wrapper wp-clearfix\">\
<a href=\"/admin/menus?{menu_q}\" class=\"nav-tab{e_on}\">Edit Menus</a>\
<a href=\"/admin/menus?{menu_q}action=locations\" class=\"nav-tab{l_on}\">Manage Locations</a></h2>",
        menu_q = menu_q,
        e_on = if locations_view { "" } else { " nav-tab-active" },
        l_on = if locations_view { " nav-tab-active" } else { "" }
    ));
    if !notice.is_empty() {
        body.push_str(&format!(
            "<div class=\"notice notice-success is-dismissible\"><p>{notice}</p></div>"
        ));
    }
    if locations_view {
        // Manage Locations tab — widefat table replaces the editor frame.
        body.push_str(&locations_html(sc, &s, &menus));
        body.push_str("</div>");
        body.push_str(MENUS_STYLE);
        body.push_str("<script src=\"/admin/wp/menu-editor.js\"></script>");
        return html_admin(&s, "Menus", &body, false, "nav-menus-php").into_response();
    }
    if !menus.is_empty() {
        // Selector bar — WP renders a plain GET form + Select button.
        let mut opts = String::new();
        for m in &menus {
            let sel = match &menu {
                Some(cur) if cur.id == m.id => " selected",
                _ => "",
            };
            opts.push_str(&format!(
                "<option value=\"{}\"{sel}>{}</option>",
                m.id,
                esc(&m.name)
            ));
        }
        body.push_str(&format!(
            "<div class=\"manage-menus\"><form method=\"get\" action=\"/admin/menus\">\
<span class=\"add-edit-delete-action\"><label for=\"select-menu-to-edit\">Select a menu to edit:</label>\
<select id=\"select-menu-to-edit\" name=\"menu\">{opts}</select>\
<input type=\"submit\" class=\"button\" value=\"Select\">\
</span><span class=\"add-new-menu-action\">or <a href=\"/admin/menus?menu=0\">create a new menu</a></span>\
</form></div>"
        ));
    }
    body.push_str("<div id=\"nav-menus-frame\" class=\"wp-clearfix\">");
    if menus.is_empty() || matches!(want, Some(0)) {
        // "create your first menu" / create view — structure column only.
        let prompt = if menus.is_empty() {
            "create your first menu"
        } else {
            "Give your menu a name, then click Create Menu."
        };
        body.push_str(&format!(
            "<div id=\"menu-management-liquid\"><div id=\"menu-management\">\
<form method=\"post\" action=\"/admin/menus/create\">\
<input type=\"hidden\" name=\"csrf\" value=\"{}\">\
<div class=\"menu-edit\"><div id=\"nav-menu-header\"><div class=\"major-publishing-actions wp-clearfix\">\
<label class=\"menu-name-label howto open-label\" for=\"menu-name\">Menu Name</label>\
<input name=\"menu-name\" id=\"menu-name\" type=\"text\" class=\"menu-name regular-text menu-item-textbox\">\
<div class=\"publishing-action\"><input type=\"submit\" class=\"button button-primary menu-save\" value=\"Create Menu\"></div>\
</div></div><div id=\"post-body\"><div id=\"post-body-content\" class=\"wp-clearfix\">\
<h3>Menu Structure</h3><p>{prompt}</p>\
</div></div></div></form></div></div>",
            s.csrf
        ));
    } else if let Some(m) = &menu {
        body.push_str(&frame_html(sc, &s, m, &menus_screen));
    }
    body.push_str("</div></div>");
    body.push_str(MENUS_STYLE);
    body.push_str("<noscript><style>.menu-item-edit-inactive .menu-item-settings{display:block}.tabs-panel-inactive{display:block}</style></noscript>");
    body.push_str("<script src=\"/admin/wp/jquery-ui.min.js\"></script><script src=\"/admin/wp/menu-editor.js\"></script>");
    html_admin(&s, "Menus", &body, false, "nav-menus-php").into_response()
}

/// Page-scoped overrides: core hides `.field-move .button-link` until its JS
/// unhides them; ours are real controls so force them visible. `.hidden` +
/// `#screen-meta-*` cover the Screen Options tab/panel (same rules the
/// comments page injects).
const MENUS_STYLE: &str = "<style>\
#menu-to-edit .field-move .button-link{display:inline}\
.quick-search-wrap input.quick-search{width:100%}\
button.item-delete{border:none;background:none;padding:0}\
#nav-menus-frame .hidden{display:none}\
#screen-meta-links{float:right;margin:0 20px 0 0}\
#screen-meta-links .show-settings{border:1px solid #c3c4c7;border-top:0;background:#f6f7f7;padding:3px 12px;border-radius:0 0 4px 4px}\
#screen-options-wrap{background:#fff;border:1px solid #c3c4c7;padding:15px 20px;box-shadow:0 2px 5px rgba(0,0,0,.15)}\
#screen-options-wrap.hidden{display:none}\
</style>";

/// Edit view: left metabox accordion + right Menu Structure column.
/// `ms` gates per-user Screen Options (advanced fields + accordion sections).
fn frame_html(sc: &SiteCtx, s: &Sess, m: &crud::Menu, ms: &crud::MenuScreen) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let mut h = String::new();
    // Standalone POST forms referenced by `form=` attributes (forms inside
    // other forms or the accordion cannot nest).
    h.push_str(&format!(
        "<form id=\"menu-del\" method=\"post\" action=\"/admin/menus/delete\">\
<input type=\"hidden\" name=\"csrf\" value=\"{0}\"><input type=\"hidden\" name=\"menu\" value=\"{1}\"></form>\
<form id=\"menu-move\" method=\"post\" action=\"/admin/menus/move\">\
<input type=\"hidden\" name=\"csrf\" value=\"{0}\"><input type=\"hidden\" name=\"menu\" value=\"{1}\"></form>\
<form id=\"menu-add\" method=\"post\" action=\"/admin/menus/add-items\">\
<input type=\"hidden\" name=\"csrf\" value=\"{0}\"><input type=\"hidden\" name=\"menu\" value=\"{1}\"></form>",
        s.csrf, m.id
    ));
    // Left column: add-item metaboxes in WP order. Sections disabled in
    // Screen Options are not rendered at all (WP removes the metabox).
    h.push_str("<div id=\"menu-settings-column\" class=\"metabox-holder\"><div class=\"accordion-container\"><ul class=\"outer-border\">");
    let mut first_box = true;
    let mut push_box = |key: &str, html: String| {
        if !ms.section(key) {
            return;
        }
        let html = if first_box {
            first_box = false;
            html.replacen("class=\"control-section accordion-section ", "class=\"control-section accordion-section open ", 1)
        } else {
            html
        };
        h.push_str(&html);
    };
    push_box("page", object_box("page", "Pages", sc, &m.lang));
    push_box("post", object_box("post", "Posts", sc, &m.lang));
    // Non-core kinds contribute one object metabox each (registry-driven).
    for def in crate::plugins::kind_defs() {
        if crate::plugins::adapter_for_kind(def.kind).is_some() {
            push_box(def.kind, object_box(def.kind, def.list_title, sc, &m.lang));
        }
    }
    push_box("custom-links", custom_link_box());
    push_box("category", category_box(sc));
    for tax in crud::list_taxonomies(&sc.db).unwrap_or_default() {
        if matches!(tax.as_str(), "category" | "post_tag" | "nav_menu") {
            continue;
        }
        let key = format!("tax-{}", tax.replace('-', "_"));
        push_box(&key, taxonomy_box(sc, &tax));
    }
    h.push_str("</ul></div></div>");
    // Right column: Menu Structure.
    let items = crud::list_items(&sc.db, m.id).unwrap_or_default();
    let mut lis = String::new();
    for (i, it) in items.iter().enumerate() {
        lis.push_str(&item_html(sc, m.id, it, items.get(i.wrapping_sub(1)), &items, ms));
    }
    if lis.is_empty() {
        lis.push_str(
            "<li class=\"menu-item-edit-inactive\"><div class=\"menu-item-settings\"><p>Drag or add items to build the menu.</p></div></li>",
        );
    }
    h.push_str(&format!(
        "<div id=\"menu-management-liquid\"><div id=\"menu-management\">\
<form id=\"update-nav-menu\" method=\"post\" action=\"/admin/menus/save\">\
<input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><input type=\"hidden\" name=\"menu\" value=\"{id}\">\
<div class=\"menu-edit\">\
<div id=\"nav-menu-header\"><div class=\"major-publishing-actions wp-clearfix\">\
<label class=\"menu-name-label howto open-label\" for=\"menu-name\">Menu Name</label>\
<input name=\"menu-name\" id=\"menu-name\" type=\"text\" class=\"menu-name regular-text menu-item-textbox\" value=\"{name}\">\
<div class=\"publishing-action\"><input type=\"submit\" class=\"button button-primary menu-save\" value=\"Save Menu\"></div>\
</div></div>\
<div id=\"post-body\"><div id=\"post-body-content\" class=\"wp-clearfix\">\
<h3>Menu Structure</h3>\
<div class=\"drag-instructions post-body-plain\"><p>Drag the items into the order you prefer. Click the arrow on the right of the item to reveal additional configuration options.</p></div>\
<ul id=\"menu-to-edit\" class=\"menu\">{lis}</ul>\
</div></div>\
</div>\
<div class=\"menu-settings\">\
<h3>Menu Settings</h3>\
<p><label class=\"menu-setting-checkbox\">\
<input type=\"checkbox\" id=\"auto-add-pages\" name=\"auto_add\" value=\"1\"{auto_checked}>\
Automatically add new top-level pages to this menu</label></p>\
</div>\
<div id=\"nav-menu-footer\"><div class=\"major-publishing-actions wp-clearfix\">\
<div class=\"publishing-action\"><input type=\"submit\" class=\"button button-primary menu-save\" value=\"Save Menu\"></div>\
<span class=\"delete-action\"><button type=\"submit\" class=\"button-link-delete menu-delete submitdelete deletion\" form=\"menu-del\" onclick=\"return confirm('You are about to permanently delete this menu.\\n\\'Cancel\\' to stop, \\'OK\\' to delete.');\">Delete Menu</button></span>\
</div></div>\
</form></div></div>",
        csrf = s.csrf,
        id = m.id,
        name = esc(&m.name),
        lis = lis,
        auto_checked = if m.auto_add { " checked" } else { "" }
    ));
    h
}

/// One accordion metabox listing publish posts of `kind` with inner
/// Most Recent | View All | Search tabs. Checkboxes post `p-{id}` +
/// `it-{id}` (type marker) through the shared `menu-add` form.
fn object_box(kind: &str, title: &str, sc: &SiteCtx, lang: &str) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let recent = crud::panel_posts(&sc.db, kind, lang, true).unwrap_or_default();
    let all = crud::panel_posts(&sc.db, kind, lang, false).unwrap_or_default();
    let lis = |rows: &[(i64, String)]| -> String {
        if rows.is_empty() {
            return "<li><p>No items.</p></li>".into();
        }
        let mut o = String::new();
        for (id, t) in rows {
            let label = if t.is_empty() { "(no title)".to_string() } else { esc(t) };
            o.push_str(&format!(
                "<li data-title=\"{dl}\"><label class=\"menu-item-title\">\
<input type=\"checkbox\" class=\"menu-item-checkbox\" name=\"p-{kind}-{id}\" value=\"1\" form=\"menu-add\"> {label}</label></li>",
                id = id,
                label = label,
                kind = kind,
                dl = esc(&t.to_lowercase())
            ));
        }
        o
    };
    tabs_box(kind, title, &lis(&recent), &lis(&all), &lis(&all))
}

/// Shared metabox chrome: tab list + three panels + Select All / Add to Menu.
fn tabs_box(key: &str, title: &str, recent: &str, all: &str, search: &str) -> String {
    format!(
        "<li class=\"control-section accordion-section add-{key}\" id=\"add-{key}\"><h3 class=\"accordion-section-title hndle\"><button type=\"button\" class=\"accordion-trigger\" aria-expanded=\"false\" aria-controls=\"add-{key}-content\"><span class=\"accordion-title\">{title}<span class=\"dashicons dashicons-arrow-down\" aria-hidden=\"true\"></span></span></button></h3>\
<div class=\"accordion-section-content\" id=\"add-{key}-content\"><div class=\"inside\"><div class=\"posttypediv\">\
<ul class=\"posttype-tabs add-menu-item-tabs\">\
<li class=\"tabs\"><a class=\"nav-tab-link\" data-type=\"tabs-panel-{key}-recent\" href=\"#\">Most Recent</a></li>\
<li><a class=\"nav-tab-link\" data-type=\"tabs-panel-{key}-all\" href=\"#\">View All</a></li>\
<li><a class=\"nav-tab-link\" data-type=\"tabs-panel-{key}-search\" href=\"#\">Search</a></li>\
</ul>\
<div id=\"tabs-panel-{key}-recent\" class=\"tabs-panel tabs-panel-active\"><ul class=\"categorychecklist form-no-clear\">{recent}</ul></div>\
<div id=\"tabs-panel-{key}-all\" class=\"tabs-panel tabs-panel-inactive\"><ul class=\"categorychecklist form-no-clear\">{all}</ul></div>\
<div id=\"tabs-panel-{key}-search\" class=\"tabs-panel tabs-panel-inactive\">\
<p class=\"quick-search-wrap\"><input type=\"search\" class=\"quick-search\" data-list=\"tabs-panel-{key}-search\" value=\"\" title=\"Search\"></p>\
<ul class=\"categorychecklist form-no-clear\">{search}</ul></div>\
<p class=\"button-controls wp-clearfix\"><span class=\"list-controls\">\
<a class=\"select-all\" href=\"#\">Select All</a></span>\
<span class=\"add-to-menu\"><button type=\"submit\" class=\"button submit-add-to-menu right\" form=\"menu-add\">Add to Menu</button></span></p>\
</div></div></div></li>"
    )
}

/// Custom Links metabox: URL + Link Text inputs, no tabs.
fn custom_link_box() -> String {
    "<li class=\"control-section accordion-section add-custom-links\" id=\"add-custom-links\"><h3 class=\"accordion-section-title hndle\"><button type=\"button\" class=\"accordion-trigger\" aria-expanded=\"false\" aria-controls=\"add-custom-links-content\"><span class=\"accordion-title\">Custom Links<span class=\"dashicons dashicons-arrow-down\" aria-hidden=\"true\"></span></span></button></h3>\
<div class=\"accordion-section-content\" id=\"add-custom-links-content\"><div class=\"inside\"><div class=\"customlinkdiv\">\
<p id=\"menu-item-url-wrap\" class=\"wp-clearfix\"><label class=\"howto\" for=\"custom-menu-item-url\">URL</label>\
<input id=\"custom-menu-item-url\" name=\"custom-url\" type=\"text\" class=\"code menu-item-textbox\" value=\"http://\" form=\"menu-add\"></p>\
<p id=\"menu-item-name-wrap\" class=\"wp-clearfix\"><label class=\"howto\" for=\"custom-menu-item-name\">Link Text</label>\
<input id=\"custom-menu-item-name\" name=\"custom-label\" type=\"text\" class=\"regular-text menu-item-textbox\" form=\"menu-add\"></p>\
<p class=\"button-controls wp-clearfix\"><span class=\"add-to-menu\">\
<button type=\"submit\" class=\"button submit-add-to-menu right\" form=\"menu-add\">Add to Menu</button></span></p>\
</div></div></div></li>"
        .to_string()
}

/// Categories metabox (terms taxonomy `category`). Same 3-tab chrome;
/// "Most Recent" = most used (count desc) to match WP's Most Used.
fn category_box(sc: &SiteCtx) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let mut terms = crud::list_terms(&sc.db, "category").unwrap_or_default();
    let lis = |rows: &[crud::TermRow]| -> String {
        if rows.is_empty() {
            return "<li><p>No items.</p></li>".into();
        }
        let mut o = String::new();
        for t in rows {
            o.push_str(&format!(
                "<li data-title=\"{dl}\"><label class=\"menu-item-title\">\
<input type=\"checkbox\" class=\"menu-item-checkbox\" name=\"p-cat-{id}\" value=\"1\" form=\"menu-add\"> {name}</label></li>",
                id = t.id,
                name = esc(&t.name),
                dl = esc(&t.name.to_lowercase())
            ));
        }
        o
    };
    let all = lis(&terms);
    terms.sort_by(|a, b| b.count.cmp(&a.count));
    let recent = lis(&terms[..terms.len().min(15)]);
    tabs_box("category", "Categories", &recent, &all, &all)
}

/// Taxonomy metabox for any non-category taxonomy (tour taxonomies, post_tag,
/// etc.). Checkbox key is `p-tax-{TAX}-{id}` where `TAX` = taxonomy slug with
/// `-`→`_`; server decodes by `_`→`-` to find the term's taxonomy.
fn taxonomy_box(sc: &SiteCtx, tax: &str) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let mut terms = crud::list_terms(&sc.db, tax).unwrap_or_default();
    let key = tax.replace('-', "_");
    let lis = |rows: &[crud::TermRow]| -> String {
        if rows.is_empty() {
            return "<li><p>No items.</p></li>".into();
        }
        let mut o = String::new();
        for t in rows {
            o.push_str(&format!(
                "<li data-title=\"{dl}\"><label class=\"menu-item-title\">\
<input type=\"checkbox\" class=\"menu-item-checkbox\" name=\"p-tax-{key}-{id}\" value=\"1\" form=\"menu-add\"> {name}</label></li>",
                key = key,
                id = t.id,
                name = esc(&t.name),
                dl = esc(&t.name.to_lowercase())
            ));
        }
        o
    };
    let all = lis(&terms);
    terms.sort_by(|a, b| b.count.cmp(&a.count));
    let recent = lis(&terms[..terms.len().min(15)]);
    let theme = crate::db::conn(&sc.db).map(|c| crate::plugins::theme_for_site(&c)).unwrap_or(&crate::plugins::DEFAULT_THEME);
    tabs_box(&format!("tax-{}", key), &tax_label_for(theme, tax).1, &recent, &all, &all)
}
/// Accordion metabox `(section-key, label)` pairs in render order — shared by
/// the Screen Options checkboxes and the apply handler. Keys match the
/// `.add-{key}` class each box renders (`tax-{escaped}` for taxonomies).
fn section_keys(sc: &SiteCtx) -> Vec<(String, String)> { // gate-allow: blocking by contract — only called inside spawn_blocking
    let mut v = vec![
        ("page".into(), "Pages".into()),
        ("post".into(), "Posts".into()),
    ];
    for def in crate::plugins::kind_defs() {
        if crate::plugins::adapter_for_kind(def.kind).is_some() {
            v.push((def.kind.into(), def.list_title.into()));
        }
    }
    v.extend([
        ("custom-links".into(), "Custom Links".into()),
        ("category".into(), "Categories".into()),
    ]);

    for tax in crud::list_taxonomies(&sc.db).unwrap_or_default() {
        if matches!(tax.as_str(), "category" | "post_tag" | "nav_menu") {
            continue;
        }
        let theme = crate::db::conn(&sc.db).map(|c| crate::plugins::theme_for_site(&c)).unwrap_or(&crate::plugins::DEFAULT_THEME);
        v.push((format!("tax-{}", tax.replace('-', "_")), tax_label_for(theme, &tax).1));
    }
    v
}

/// WP `#screen-meta` + `#screen-meta-links` for nav-menus.php: advanced menu
/// property toggles (checked = field visible) + accordion section toggles.
/// Fields default hidden (WP parity: advanced properties start unchecked).
fn screen_options_html(sc: &SiteCtx, s: &Sess, ms: &crud::MenuScreen) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    // (key, label) — checkbox value doubles as the `.field-{key}` toggle class.
    const FIELDS: [(&str, &str); 5] = [
        ("title_attr", "Title Attribute"),
        ("target", "Link Target"),
        ("classes", "CSS Classes"),
        ("xfn", "Link Relationship (XFN)"),
        ("description", "Description"),
    ];
    let mut fields = String::new();
    for (key, label) in FIELDS {
        fields.push_str(&format!(
            "<label for=\"show-{key}\"><input class=\"hide-field-tog\" name=\"show-{key}\" type=\"checkbox\" id=\"show-{key}\" value=\"{key}\"{}>{label}</label>",
            if ms.field(key) { " checked" } else { "" }
        ));
    }
    let mut sections = String::new();
    for (key, label) in section_keys(sc) {
        sections.push_str(&format!(
            "<label for=\"sec-{key}\"><input class=\"hide-acc-tog\" name=\"sec-{key}\" type=\"checkbox\" id=\"sec-{key}\" value=\"{key}\"{}>{label}</label>",
            if ms.section(&key) { " checked" } else { "" }
        ));
    }
    format!(
        r#"<div id="screen-meta" class="metabox-prefs">
<div id="screen-options-wrap" class="hidden" tabindex="-1" aria-label="Screen Options Tab">
<form id="adv-settings" method="post" action="/admin/menus/screen">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="back" value="/admin/menus">
<fieldset class="metabox-prefs view-options">
<legend>Show advanced menu properties</legend>
{fields}
</fieldset>
<fieldset class="metabox-prefs">
<legend>Show on screen</legend>
{sections}
</fieldset>
<p class="submit"><input type="submit" class="button button-primary" id="screen-options-apply" value="Apply"></p>
</form>
</div>
</div>
<div id="screen-meta-links">
<div id="screen-options-link-wrap" class="hide-if-no-js screen-meta-toggle">
<button type="button" id="show-settings-link" class="button show-settings" aria-controls="screen-options-wrap" aria-expanded="false">Screen Options</button>
</div>
</div>"#,
        csrf = s.csrf,
        fields = fields,
        sections = sections
    )
}

/// Manage Locations tab — widefat table of `theme_locations_available`
/// rows, each with an Assigned Menu dropdown; one Save Changes posts the
/// whole grid (`assign-{location}` keys, empty = unassigned).
fn locations_html(sc: &SiteCtx, s: &Sess, menus: &[crud::Menu]) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let locs = crud::available_locations(&sc.db);
    let assigned = crud::list_locations(&sc.db, &locs).unwrap_or_default();
    /* Per-lang assignment grid: each (location, lang) pair gets its own
       select — Polylang binds different menus to the same location per
       language. Langs come from the menus on the site plus the row's
       stored lang. */
    let mut rows = String::new();
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for (loc, lang, menu_id) in &assigned {
        if !seen.insert((loc.clone(), lang.clone())) {
            continue;
        }
        let key = if lang.is_empty() {
            format!("assign-{loc}")
        } else {
            format!("assign-{loc}-{lang}")
        };
        let mut opts = String::from("<option value=\"0\">— Select —</option>");
        for m in menus {
            let sel = if *menu_id == Some(m.id) { " selected" } else { "" };
            opts.push_str(&format!("<option value=\"{}\"{sel}>{}</option>", m.id, esc(&m.name)));
        }
        let loc_label = if lang.is_empty() {
            esc(loc)
        } else {
            format!("{} ({})", esc(loc), esc(lang))
        };
        rows.push_str(&format!(
            "<tr><td class=\"location column-location\">{loc_label}</td>\
<td class=\"menu column-menu\"><select name=\"{key}\">{opts}</select></td></tr>"
        ));
    }
    if rows.is_empty() {
        rows.push_str("<tr><td colspan=\"2\">No locations declared by the active theme.</td></tr>");
    }
    format!(
        "<form method=\"post\" action=\"/admin/menus/locations\">\
<input type=\"hidden\" name=\"csrf\" value=\"{csrf}\">\
<table class=\"widefat fixed\" id=\"menu-locations-table\">\
<thead><tr><th class=\"manage-column column-location\">Theme Location</th>\
<th class=\"manage-column column-menu\">Assigned Menu</th></tr></thead>\
<tbody>{rows}</tbody></table>\
<p class=\"submit\"><input type=\"submit\" class=\"button button-primary\" value=\"Save Changes\"></p>\
</form>",
        csrf = s.csrf
    )
}


/// Depth of an item = length of its parent chain (cycle-guarded).
fn depth_of(it: &crud::MenuItem, all: &[crud::MenuItem]) -> usize { // gate-allow: blocking by contract — only called inside spawn_blocking
    let mut d = 0;
    let mut p = it.parent_id;
    while let Some(pid) = p {
        d += 1;
        match all.iter().find(|x| x.id == pid) {
            Some(par) => p = par.parent_id,
            None => break,
        }
        if d > 32 {
            break;
        }
    }
    d
}

/// WP type label for `.item-type` (Custom Link / Page / Post / Tour / Category).
fn type_label(sc: &SiteCtx, it: &crud::MenuItem) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    match it.item_type.as_str() {
        "post_type" => match it.post_kind.as_str() {
            "page" => "Page".into(),
            "post" => "Post".into(),
            "tour" => "Tour".into(),
            k if !k.is_empty() => {
                let mut c = k.chars();
                format!("{}{}", c.next().unwrap().to_uppercase(), c.as_str())
            }
            _ => "Post".into(),
        },
        "taxonomy" => {
            if it.term_taxonomy.is_empty() {
                "Category".into()
            } else {
                let theme = crate::db::conn(&sc.db).map(|c| crate::plugins::theme_for_site(&c)).unwrap_or(&crate::plugins::DEFAULT_THEME);
                tax_label_for(theme, &it.term_taxonomy).1
            }
        }
        "custom" => "Custom Link".into(),
        "lang-switcher" => "Language".into(),
        _ => "Custom Link".into(),
    }
}

/// "Original:" link target — public permalink for posts, term editor for
/// taxonomy items (WP uses the archive URL; no public term path exists here).
fn original(it: &crud::MenuItem, sc: &SiteCtx) -> Option<(String, String)> { // gate-allow: blocking by contract — only called inside spawn_blocking
    match it.item_type.as_str() {
        "post_type" if it.post_id.is_some() => Some((
            public_path(sc, &it.post_lang, &it.post_kind, &it.post_slug),
            if it.post_title.is_empty() { "(no title)".into() } else { it.post_title.clone() },
        )),
        "taxonomy" if it.term_id.is_some() => Some((
            format!("/admin/tax/edit/{}", it.term_id.unwrap_or(0)),
            it.term_name.clone(),
        )),
        "custom" if !it.url.is_empty() => Some((it.url.clone(), it.url.clone())),
        _ => None,
    }
}

/// One `li.menu-item` block: `.menu-item-bar` handle + `.menu-item-settings`
/// with the advanced fields WP puts under "Show advanced menu properties" —
/// each wrapped in `.field-{key}` and `hidden` unless enabled in Screen
/// Options (`ms`), so the JS toggle can flip them without a reload.
fn item_html(sc: &SiteCtx, menu_id: i64, it: &crud::MenuItem, prev: Option<&crud::MenuItem>, all: &[crud::MenuItem], ms: &crud::MenuScreen) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let d = depth_of(it, all);
    let sub = if d > 0 {
        "<span class=\"is-submenu\">sub item</span>"
    } else {
        ""
    };
    // WP: an unset navigation label renders the object's own title —
    // imported taxonomy/post items routinely have empty `label`.
    let label = if !it.label.is_empty() {
        esc(&it.label)
    } else if !it.term_name.is_empty() {
        esc(&it.term_name)
    } else if !it.post_title.is_empty() {
        esc(&it.post_title)
    } else {
        "(no title)".to_string()
    };
    // "Original:" line.
    let orig = original(it, sc)
        .map(|(href, title)| {
            format!(
                "<p class=\"link-to-original\">Original: <a href=\"{}\" target=\"_blank\">{}</a></p>",
                esc(&href),
                esc(&title)
            )
        })
        .unwrap_or_default();
    // Move links — POST buttons through the shared menu-move form; JS
    // intercepts for client-side reorder.
    let mv = |dir: &str, text: &str, show: bool| -> String {
        let hid = if show { "" } else { " hidden" };
        format!(
            "<button type=\"submit\" class=\"button-link\" data-dir=\"{dir}\"{hid} form=\"menu-move\" formaction=\"/admin/menus/move?menu={menu}&amp;id={id}&amp;dir={dir}\">{text}</button>",
            dir = dir,
            hid = hid,
            menu = menu_id,
            id = it.id,
            text = esc(text)
        )
    };
    let parent_label = it
        .parent_id
        .and_then(|pid| all.iter().find(|x| x.id == pid))
        .map(|p| if p.label.is_empty() { "(no title)".to_string() } else { p.label.clone() })
        .unwrap_or_default();
    let moves = format!(
        "{} <span aria-hidden=\"true\">|</span> {} <span aria-hidden=\"true\">|</span> {} <span aria-hidden=\"true\">|</span> {} <span aria-hidden=\"true\">|</span> {}",
        mv("up", "Up one", true),
        mv("down", "Down one", true),
        mv("under", &format!("Under {}", prev.map(|p| if p.label.is_empty() { "(no title)".to_string() } else { p.label.clone() }).unwrap_or_default()), true),
        mv("out", &format!("Out from under {}", parent_label), it.parent_id.is_some()),
        mv("top", "To the top", true)
    );
    // URL field: visible input for custom links, hidden carry-over otherwise
    // so non-custom items keep their stored url untouched.
    let url_field = if it.item_type == "custom" {
        format!(
            "<p class=\"field-url description description-wide\"><label>URL<br>\
<input type=\"text\" class=\"widefat code edit-menu-item-url\" name=\"url-{0}\" value=\"{1}\"></label></p>",
            it.id,
            esc(&it.url)
        )
    } else {
        format!("<input type=\"hidden\" name=\"url-{}\" value=\"{}\">", it.id, esc(&it.url))
    };
    // Advanced menu properties — WP class names so the vendored nav-menus
    // CSS lays them out identically; `.field-{key}` is the JS-toggle class.
    let adv = |key: &str, wp_cls: &str, inner: String| -> String {
        let hid = if ms.field(key) { "" } else { " hidden" };
        format!("<p class=\"field-{key} {wp_cls}{hid}\">{inner}</p>")
    };
    let blank_checked = if it.target == "_blank" { " checked" } else { "" };
    let adv_fields = format!(
        "{}{}{}{}{}",
        adv(
            "title_attr",
            "field-attr-title description description-thin",
            format!(
                "<label for=\"edit-menu-item-attr-title-{id}\">Title Attribute<br>\
<input type=\"text\" id=\"edit-menu-item-attr-title-{id}\" class=\"widefat edit-menu-item-attr-title\" name=\"title_attr-{id}\" value=\"{v}\"></label>",
                id = it.id,
                v = esc(&it.title_attr)
            ),
        ),
        adv(
            "target",
            "field-target description description-thin",
            format!(
                "<label for=\"edit-menu-item-target-{id}\">\
<input type=\"checkbox\" id=\"edit-menu-item-target-{id}\" name=\"target-{id}\" value=\"_blank\"{blank_checked}>\
Open link in a new tab</label>",
                id = it.id,
                blank_checked = blank_checked
            ),
        ),
        adv(
            "classes",
            "field-css-classes description description-thin",
            format!(
                "<label for=\"edit-menu-item-classes-{id}\">CSS Classes (optional)<br>\
<input type=\"text\" id=\"edit-menu-item-classes-{id}\" class=\"widefat code edit-menu-item-classes\" name=\"classes-{id}\" value=\"{v}\"></label>",
                id = it.id,
                v = esc(&it.classes)
            ),
        ),
        adv(
            "xfn",
            "field-xfn description description-thin",
            format!(
                "<label for=\"edit-menu-item-xfn-{id}\">Link Relationship (XFN)<br>\
<input type=\"text\" id=\"edit-menu-item-xfn-{id}\" class=\"widefat code edit-menu-item-xfn\" name=\"xfn-{id}\" value=\"{v}\"></label>",
                id = it.id,
                v = esc(&it.xfn)
            ),
        ),
        adv(
            "description",
            "field-description description description-wide",
            format!(
                "<label for=\"edit-menu-item-description-{id}\">Description<br>\
<textarea id=\"edit-menu-item-description-{id}\" class=\"widefat edit-menu-item-description\" name=\"description-{id}\" rows=\"3\" cols=\"20\">{v}</textarea>\
<span class=\"description\">The description will be displayed in the menu if the active theme supports it.</span></label>",
                id = it.id,
                v = esc(&it.description)
            ),
        )
    );
    let parent_v = it.parent_id.unwrap_or(0);
    format!(
        "<li id=\"menu-item-{id}\" class=\"menu-item menu-item-depth-{d} menu-item-edit-inactive\" data-item-id=\"{id}\">\
<dl class=\"menu-item-bar\"><dt class=\"menu-item-handle ui-sortable-handle\">\
<span class=\"item-title\"><span class=\"menu-item-title\">{label}</span>{sub}</span>\
<span class=\"item-controls\"><span class=\"item-type\">{ty}</span><a class=\"item-edit\" id=\"edit-{id}\" href=\"#\"><span class=\"screen-reader-text\">Edit</span></a></span>\
</dt></dl>\
<div class=\"menu-item-settings wp-clearfix\" id=\"menu-item-settings-{id}\">\
<p class=\"description description-wide\"><label>Navigation Label<br>\
<input type=\"text\" class=\"widefat code edit-menu-item-title\" name=\"title-{id}\" value=\"{label}\"></label></p>\
{url_field}\
{adv_fields}\
<div class=\"field-move\"><label>Move</label> {moves}</div>\
{orig}\
<button type=\"submit\" name=\"rm-{id}\" value=\"1\" class=\"item-delete submitdelete deletion button-link\">Remove</button>\
<a class=\"item-cancel submitcancel\" href=\"#\">Cancel</a>\
<input type=\"hidden\" name=\"ord-{id}\" value=\"{ord}\">\
<input type=\"hidden\" name=\"parent-{id}\" value=\"{parent_v}\">\
<input type=\"hidden\" name=\"del-{id}\" class=\"menu-item-del\" value=\"0\">\
</div></li>",
        id = it.id,
        d = d,
        label = label,
        sub = sub,
        ty = esc(&type_label(sc, it)),
        url_field = url_field,
        adv_fields = adv_fields,
        moves = moves,
        orig = orig,
        ord = it.ord,
        parent_v = parent_v
    )
}

fn csrf_ok(s: &Sess, map: &HashMap<String, String>) -> bool {
    ct_eq(s.csrf.as_bytes(), map.get("csrf").map(|x| x.as_str()).unwrap_or("").as_bytes())
}

/// `POST /admin/menus/create` — name → redirect to the new menu's edit view.
pub(crate) async fn menus_create(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let name = map.get("menu-name").cloned().unwrap_or_default();
    let name = name.trim();
    if name.is_empty() {
        return Redirect::to("/admin/menus?menu=0").into_response();
    }
    // rusqlite is sync — offload conn+insert to a blocking thread.
    let sc2 = sc.clone();
    let name = name.to_string();
    let created = tokio::task::spawn_blocking(move || {
        let Ok(c) = crate::db::conn(&sc2.db) else {
            return Err(());
        };
        // New menus default to the site's primary language (N-lang — was
        // hardwired "fr").
        let primary = crate::lang::config(&c).primary;
        let r = crud::create_menu(&sc2.db, &name, &primary);
        if r.is_ok() {
            crate::admin::admin::bust_cache(&sc2);
        }
        r.map_err(|_| ())
    })
    .await
    .unwrap_or(Err(()));
    match created {
        Ok(id) => Redirect::to(&format!("/admin/menus?menu={id}&msg=created")).into_response(),
        Err(_) => Redirect::to("/admin/menus").into_response(),
    }
}

/// `POST /admin/menus/delete` — items + menu row in one tx.
pub(crate) async fn menus_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let id = map.get("menu").and_then(|v| v.parse().ok()).unwrap_or(0);
    // rusqlite is sync — offload delete + cache bust to a blocking thread.
    let sc2 = sc.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let _ = crud::delete_menu(&sc2.db, id);
        crate::admin::admin::bust_cache(&sc2);
    })
    .await;
    Redirect::to("/admin/menus?msg=deleted").into_response()
}

/// `POST /admin/menus/add-items` — append checked objects (+ one custom
/// link) as last top-level items, in check order. Raw body keeps field
/// order; `p-{id}` = checked object, `it-{id}` = its type marker.
pub(crate) async fn menus_add(
    State(app): State<App>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    let pairs: Vec<(String, String)> =
        serde_urlencoded::from_bytes(body.as_bytes()).unwrap_or_default();
    let get = |k: &str| pairs.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone());
    if !ct_eq(s.csrf.as_bytes(), get("csrf").as_deref().unwrap_or("").as_bytes()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let menu: i64 = get("menu").and_then(|v| v.parse().ok()).unwrap_or(0);
    if menu <= 0 {
        return Redirect::to("/admin/menus").into_response();
    }
    // First pass: checked ids in body order. Keys are `p-{type}-{id}` —
    // embedding the type in the name prevents post-id/term-id collisions.
    // `p-tax-{slug}-{id}` carries taxonomy slug (with `-`→`_` escaped); `p-{type}-{id}`
    // covers post kinds (`page`,`post`,`tour`,…) plus `cat` for category terms.
    let mut order: Vec<(String, i64, bool)> = Vec::new(); // (key, id, is_taxonomy)
    for (k, _v) in &pairs {
        let Some(rest) = k.strip_prefix("p-") else { continue };
        if let Some(tr) = rest.strip_prefix("tax-") {
            if let Some((tax_e, id_s)) = tr.rsplit_once('-') {
                if let Ok(id) = id_s.parse::<i64>() {
                    let tax = tax_e.replace('_', "-");
                    if !order.iter().any(|(t, i, _)| t == &tax && *i == id) {
                        order.push((tax, id, true));
                    }
                }
            }
            continue;
        }
        let Some((ty, id_s)) = rest.split_once('-') else { continue };
        let Ok(id) = id_s.parse::<i64>() else { continue };
        if !order.iter().any(|(t, i, _)| t == ty && *i == id) {
            order.push((ty.to_string(), id, ty == "cat"));
        }
    }
    // Custom link fields ride the same form; add one when a URL was typed.
    let custom_url = get("custom-url").map(|u| u.trim().to_string()).filter(|u| {
        // Scheme gate (audit M2): `javascript:`/`data:` never enter the
        // DB — WP's esc_url equivalent, enforced at write AND render.
        !u.is_empty() && u != "http://" && crate::url_util::safe_href(u)
    });
    let custom_label = get("custom-label").map(|l| l.trim().to_string()).unwrap_or_default();
    // rusqlite is sync — offload label lookups + insert to a blocking thread.
    let sc2 = sc.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let mut items: Vec<crud::NewItem> = Vec::new();
        for (_ty, id, is_tax) in order {
            if is_tax {
                if let Ok(Some((name, _))) = crud::term_label(&sc2.db, id) {
                    items.push(crud::NewItem {
                        label: name,
                        url: String::new(),
                        object_id: Some(id),
                        item_type: "taxonomy".into(),
                    });
                }
            } else if let Ok(Some((title, _kind))) = crud::post_label(&sc2.db, id) {
                items.push(crud::NewItem {
                    label: title,
                    url: String::new(),
                    object_id: Some(id),
                    item_type: "post_type".into(),
                });
            }
        }
        if let Some(url) = custom_url {
            items.push(crud::NewItem {
                label: if custom_label.is_empty() { url.clone() } else { custom_label },
                url,
                object_id: None,
                item_type: "custom".into(),
            });
        }
        let _ = crud::add_items(&sc2.db, menu, &items);
        crate::admin::admin::bust_cache(&sc2);
    })
    .await;
    Redirect::to(&format!("/admin/menus?menu={menu}&msg=added")).into_response()
}

/// `POST /admin/menus/save` — rename + auto-add flag + structure + deletions,
/// ONE tx. Item keys: `ord-{id}`, `parent-{id}`, `title-{id}`, `url-{id}`,
/// `del-{id}` (+ `rm-{id}` = no-JS Remove) and the advanced-property keys
/// `title_attr-{id}`, `target-{id}`, `classes-{id}`, `xfn-{id}`,
/// `description-{id}`.
pub(crate) async fn menus_save(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let menu: i64 = map.get("menu").and_then(|v| v.parse().ok()).unwrap_or(0);
    if menu <= 0 {
        return Redirect::to("/admin/menus").into_response();
    }
    // rusqlite is sync — offload item build + tree save + cache bust to a
    // blocking thread.
    let sc2 = sc.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let mut items = Vec::new();
        for k in map.keys() {
            let Some(id) = k.strip_prefix("ord-").and_then(|x| x.parse::<i64>().ok()) else {
                continue;
            };
            let deleted = map.get(&format!("del-{id}")).map(|v| v == "1").unwrap_or(false)
                || map.contains_key(&format!("rm-{id}"));
            items.push(crud::ItemForm {
                id,
                ord: map.get(k).and_then(|v| v.parse().ok()).unwrap_or(0),
                parent_id: map
                    .get(&format!("parent-{id}"))
                    .and_then(|v| v.parse::<i64>().ok())
                    .filter(|p| *p > 0),
                label: map.get(&format!("title-{id}")).cloned().unwrap_or_default(),
                // Unsafe submitted URL: keep the *submitted* value, never
                // silently blank it (the save would otherwise destroy an
                // existing URL). Render-time `safe_href_or_hash` already
                // neutralizes any unsafe scheme on output.
                url: map.get(&format!("url-{id}")).cloned().unwrap_or_default(),
                title_attr: map.get(&format!("title_attr-{id}")).cloned().unwrap_or_default(),
                target: map
                    .get(&format!("target-{id}"))
                    .filter(|v| v.as_str() == "_blank")
                    .cloned()
                    .unwrap_or_default(),
                classes: map.get(&format!("classes-{id}")).cloned().unwrap_or_default(),
                xfn: map.get(&format!("xfn-{id}")).cloned().unwrap_or_default(),
                description: map.get(&format!("description-{id}")).cloned().unwrap_or_default(),
                delete: deleted,
            });
        }
        let _ = crud::save_tree(
            &sc2.db,
            menu,
            map.get("menu-name").map(|s| s.as_str()),
            &items,
            Some(map.contains_key("auto_add")),
        );
        crate::admin::admin::bust_cache(&sc2);
    })
    .await;
    Redirect::to(&format!("/admin/menus?menu={menu}&msg=saved")).into_response()
}

/// `POST /admin/menus/move` — server-side fallback for the move links:
/// `dir` = up|down|under|out|top, `id` = item. Always correct without JS.
pub(crate) async fn menus_move(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let menu: i64 = map
        .get("menu")
        .or_else(|| q.get("menu"))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let id: i64 = q.get("id").and_then(|v| v.parse().ok()).unwrap_or(0);
    let dir = q.get("dir").cloned().unwrap_or_default();
    // rusqlite is sync — offload move + cache bust to a blocking thread.
    let sc2 = sc.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let _ = crud::move_item(&sc2.db, menu, id, &dir);
        crate::admin::admin::bust_cache(&sc2);
    })
    .await;
    Redirect::to(&format!("/admin/menus?menu={menu}&msg=moved")).into_response()
}

/// `POST /admin/menus/locations` — Manage Locations "Save Changes". Every
/// declared location posts an `assign-{location}` select; value 0/absent =
/// unassigned. Redirects back to the tab preserving `?menu`.
pub(crate) async fn menus_locations_save(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let menu_q = q
        .get("menu")
        .and_then(|v| v.parse::<i64>().ok())
        .map(|id| format!("menu={id}&"))
        .unwrap_or_default();
    // rusqlite is sync — offload location read + assignment save to a
    // blocking thread.
    let sc2 = sc.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let locs = crud::available_locations(&sc2.db);
        /* Form posts `assign-{loc}` or `assign-{loc}-{lang}` selects; the lang
           suffix splits on the LAST '-' segment so locations containing '-'
           stay intact. */
        let mut assignments: Vec<(String, String, Option<i64>)> = Vec::new();
        for loc in &locs {
            let suffixes: Vec<String> = std::iter::once(String::new())
                .chain(
                    map.keys()
                        .filter_map(|k| {
                            k.strip_prefix(&format!("assign-{loc}-"))
                                .map(|s| s.to_string())
                        }),
                )
                .collect();
            for suf in suffixes {
                let key = if suf.is_empty() {
                    format!("assign-{loc}")
                } else {
                    format!("assign-{loc}-{suf}")
                };
                let mid = map
                    .get(&key)
                    .and_then(|v| v.parse::<i64>().ok())
                    .filter(|id| *id > 0);
                assignments.push((loc.clone(), suf, mid));
            }
        }
        let _ = crud::save_locations(&sc2.db, &assignments);
        crate::admin::admin::bust_cache(&sc2);
    })
    .await;
    Redirect::to(&format!("/admin/menus?{menu_q}action=locations&msg=locations")).into_response()
}

/// `POST /admin/menus/screen` — Screen Options "Apply": per-user advanced
/// field visibility + accordion section visibility as JSON in
/// `settings.menus_screen_{user_id}`.
pub(crate) async fn menus_screen_save(
    State(app): State<App>,
    headers: HeaderMap,
    Form(map): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !csrf_ok(&s, &map) {
        return StatusCode::FORBIDDEN.into_response();
    }
    // rusqlite is sync — offload section list + screen-options save to a
    // blocking thread.
    let sc2 = sc.clone();
    let uid = s.user_id;
    tokio::task::spawn_blocking(move || {
        let fields: Vec<String> = ["title_attr", "target", "classes", "xfn", "description"]
            .iter()
            .filter(|k| map.contains_key(&format!("show-{k}")))
            .map(|k| k.to_string())
            .collect();
        let hidden: Vec<String> = section_keys(&sc2)
            .iter()
            .filter(|(key, _)| !map.contains_key(&format!("sec-{key}")))
            .map(|(key, _)| key.clone())
            .collect();
        let _ = crud::MenuScreen::save(&sc2.db, uid, &crud::MenuScreen { fields, hidden });
        let back = list::back_or(&map, "/admin/menus");
        let sep = if back.contains('?') { "&" } else { "?" };
        Redirect::to(&format!("{back}{sep}msg=screen")).into_response()
    })
    .await
    .unwrap_or_else(|e| {
        eprintln!("menus_screen_save join: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}
