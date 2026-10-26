use super::*;
#[derive(Deserialize)]
pub(crate) struct TaxForm {
    csrf: String,
    #[serde(default)]
    id: Option<i64>,
    tax: String,
    name: String,
    slug: String,
    #[serde(default)]
    description: String,
    /// Quick Edit doesn't submit `parent` — `None` keeps the stored value.
    #[serde(default)]
    parent: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct NewFilterForm {
    csrf: String,
    label: String,
    slug: String,
}

fn tax_labels(tax: &str) -> (String, String) {
    let (pl, sg) = tax_label(tax);
    (pl.to_string(), sg)
}

/// Custom tour taxonomies created via "Add New Filter", stored as
/// settings.tour_custom_taxs = {slug: label} (tourmaster's
/// `tourmaster_custom_tour_taxs` option equivalent; hierarchy always on).
fn custom_taxs(db: &std::path::Path) -> std::collections::BTreeMap<String, String> {
    crud::setting_json_map(db, "tour_custom_taxs")
}

fn new_filter_html(s: &Sess) -> String {
    format!(
        r#"{chrome}<div class="wrap"><h1>Add New Filter</h1>
<p>Create a custom tour taxonomy (tourmaster filter). It appears under Tour → menu and on the tour edit screen.</p>
<form method="post" action="/admin/tax/new-filter" class="form-table">
<input type="hidden" name="csrf" value="{csrf}">
<table class="form-table"><tbody>
<tr><th scope="row"><label for="nf-label">Label</label></th>
<td><input id="nf-label" name="label" type="text" class="regular-text" required></td></tr>
<tr><th scope="row"><label for="nf-slug">Slug</label></th>
<td><input id="nf-slug" name="slug" type="text" class="regular-text" placeholder="auto from label"></td></tr>
</tbody></table>
<p class="submit"><button class="button button-primary">Add Filter</button></p>
</form></div>"#,
        chrome = chrome("tours", s),
        csrf = s.csrf,
    )
}

pub(crate) async fn tax_new_filter_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, _sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    html_admin(&s, "Add New Filter", &new_filter_html(&s), true, "edit-tags").into_response()
}

pub(crate) async fn tax_new_filter(State(app): State<App>, headers: HeaderMap, Form(f): Form<NewFilterForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let slug = if f.slug.trim().is_empty() { crud::slugify(&f.label) } else { f.slug.trim().to_string() };
    if slug.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let mut taxs = custom_taxs(&sc.db);
    taxs.insert(slug.clone(), f.label.trim().to_string());
    let _ = crud::setting_json_put_map(&sc.db, "tour_custom_taxs", &taxs);
    Redirect::to(&format!("/admin/tax/{slug}")).into_response()
}
#[derive(Deserialize)]
pub(crate) struct TaxDelete {
    csrf: String,
    id: i64,
    tax: String,
}

/// Non-hierarchical taxonomies (WP: `post_tag`) show no Parent field and get
/// the "most used" cloud instead. Everything else is hierarchical.
fn hierarchical(tax: &str) -> bool {
    tax != "post_tag"
}

/// URL-encode a path/query segment (`&`/`=`/spaces) for list links.
fn urlenc(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            o.push(b as char);
        } else {
            o.push_str(&format!("%{b:02X}"));
        }
    }
    o
}

/// `<div class="tablenav-pages">…` pager for the terms table — same markup
/// as `list::Pager` but honors the per-user per-page setting.
fn term_pager(base: &str, search_q: &str, page: i64, per_page: i64, total: i64) -> String {
    let pages = ((total + per_page - 1) / per_page).max(1);
    let extra = if search_q.is_empty() { String::new() } else { format!("&s={}", urlenc(search_q)) };
    let link = |p: i64| format!("{base}?paged={p}{extra}");
    let mut out = format!(
        "<div class=\"tablenav-pages\"><span class=\"displaying-num\">{total} items</span><span class=\"pagination-links\">"
    );
    if page <= 1 {
        out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">«</span>");
        out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">‹</span>");
    } else {
        out.push_str(&format!("<a class=\"first-page button\" href=\"{}\"><span aria-hidden=\"true\">«</span></a>", link(1)));
        out.push_str(&format!("<a class=\"prev-page button\" href=\"{}\"><span aria-hidden=\"true\">‹</span></a>", link(page - 1)));
    }
    out.push_str(&format!(
        "<span class=\"paging-input\"><label class=\"screen-reader-text\">Current Page</label><input class=\"current-page\" type=\"text\" name=\"paged\" value=\"{page}\" size=\"1\" disabled> of <span class=\"total-pages\">{pages}</span></span>"
    ));
    if page >= pages {
        out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">›</span>");
        out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">»</span>");
    } else {
        out.push_str(&format!("<a class=\"next-page button\" href=\"{}\"><span aria-hidden=\"true\">›</span></a>", link(page + 1)));
        out.push_str(&format!("<a class=\"last-page button\" href=\"{}\"><span aria-hidden=\"true\">»</span></a>", link(pages)));
    }
    out.push_str("</span></div>");
    out
}

/// Parent `<select>` options for one hierarchical taxonomy: DFS so children
/// print `&nbsp;`-indented under their parent. `desc` (self + descendants)
/// are rendered `disabled` — WP's cycle guard.
fn parent_opts(all: &[crud::TermRow], cur: i64, desc: &[i64], emitted: &mut std::collections::HashSet<i64>, out: &mut String) {
    let emit = |t: &crud::TermRow, depth: usize, emitted: &mut std::collections::HashSet<i64>, out: &mut String| {
        emitted.insert(t.id);
        let mut attrs = String::new();
        if desc.contains(&t.id) {
            attrs.push_str(" disabled");
        }
        if t.id == cur {
            attrs.push_str(" selected");
        }
        out.push_str(&format!(
            "<option value=\"{}\"{attrs}>{}{}</option>",
            t.id,
            "&nbsp;".repeat(depth),
            esc(&t.name)
        ));
    };
    for t in all.iter().filter(|t| t.parent_id == 0) {
        emit(t, 0, emitted, out);
        emit_kids(t.id, 1, all, cur, desc, emitted, out);
    }
    // Orphans / rows reachable only through a broken cycle still need an
    // option, else the <select> drops them silently.
    let orphans: Vec<i64> = all.iter().filter(|t| !emitted.contains(&t.id)).map(|t| t.id).collect();
    for id in orphans {
        if let Some(t) = all.iter().find(|t| t.id == id) {
            emit(t, 0, emitted, out);
        }
    }
}

fn emit_kids(pid: i64, depth: usize, all: &[crud::TermRow], cur: i64, desc: &[i64], emitted: &mut std::collections::HashSet<i64>, out: &mut String) {
    if depth > 16 {
        return;
    }
    for t in all.iter().filter(|t| t.parent_id == pid) {
        emitted.insert(t.id);
        let mut attrs = String::new();
        if desc.contains(&t.id) {
            attrs.push_str(" disabled");
        }
        if t.id == cur {
            attrs.push_str(" selected");
        }
        out.push_str(&format!(
            "<option value=\"{}\"{attrs}>{}{}</option>",
            t.id,
            "&nbsp;".repeat(depth),
            esc(&t.name)
        ));
        emit_kids(t.id, depth + 1, all, cur, desc, emitted, out);
    }
}

pub(crate) fn tax_page_html(db: &std::path::Path, s: &Sess, tax: &str, edit: Option<&crud::TermRow>, search_q: &str, page: i64) -> String {
    // WP term screens all share one table; only the labels change.
    // category/post_tag are the WP defaults, everything else = tour taxonomy.
    let (title, noun) = tax_labels(tax);
    let search_val = esc(search_q);
    let opts = crud::screen_opts(db, s.user_id, tax);
    let per_page = opts.per_page.clamp(1, 500);
    let all_terms = crud::list_terms(db, tax).unwrap_or_default();
    let mut terms = all_terms.clone();
    if !search_q.is_empty() {
        let q = search_q.to_lowercase();
        terms.retain(|t| t.name.to_lowercase().contains(&q) || t.slug.to_lowercase().contains(&q));
    }

    let total = terms.len() as i64;
    let pages = ((total + per_page - 1) / per_page).max(1);
    let page = page.clamp(1, pages);
    let start = ((page - 1) * per_page) as usize;
    let shown: Vec<crud::TermRow> = terms.iter().skip(start).take(per_page as usize).cloned().collect();

    let col = |name: &str| if opts.hide.iter().any(|h| h == name) { " hidden" } else { "" };
    let list_base = if tax_menu(tax) == "tours" { "/admin/tours" } else { "/admin/posts" };

    let mut rows = String::new();
    for t in &shown {
        let desc = if t.description.is_empty() { "—".into() } else { esc(&t.description) };
        rows.push_str(&format!(
            r##"<tr id="term-{id}" data-slug="{slug}" data-name="{name}" data-desc="{desc_raw}">
<th scope="row" class="check-column"><input type="checkbox" name="d-{id}" value="1"></th>
<td class="name column-name has-row-actions column-primary"><strong><a class="row-title" href="/admin/tax/edit/{id}">{name}</a></strong>
<div class="row-actions">
<span class="edit"><a href="/admin/tax/edit/{id}">Edit</a> | </span>
<span class="inline hide-if-no-js"><a href="#edit" class="editinline">Quick Edit</a> | </span>
<span class="delete"><form method="post" action="/admin/tax/delete" style="display:inline"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{id}"><input type="hidden" name="tax" value="{tax}"><button class="button-link">Delete</button></form> | </span>
<span class="view"><a href="{list}?tax={tax}&amp;term={id}">View</a></span>
</div></td>
<td class="description column-description{desc_col}">{desc}</td>
<td class="slug column-slug{slug_col}">{slug}</td>
<td class="posts column-posts{count_col}"><a href="{list}?tax={tax}&amp;term={id}">{count}</a></td>
</tr>"##,
            id = t.id,
            name = esc(&t.name),
            slug = esc(&t.slug),
            count = t.count,
            desc = desc,
            desc_raw = esc(&t.description),
            csrf = s.csrf,
            tax = tax,
            list = list_base,
            desc_col = col("description"),
            slug_col = col("slug"),
            count_col = col("count"),
        ));
    }
    if rows.is_empty() {
        rows.push_str(r#"<tr class="no-items"><td class="colspanchange" colspan="5">No items found.</td></tr>"#);
    }

    // Cycle guard: on Edit, self + every descendant are unsafe parents —
    // render them disabled (the server re-checks in save_term).
    let desc_ids: Vec<i64> = edit
        .map(|e| crud::descendants(db, e.id).unwrap_or_default())
        .unwrap_or_default();
    let cur_parent = edit.map(|e| e.parent_id).unwrap_or(0);
    let mut parent_options = String::from(r#"<option value="0">None</option>"#);
    if hierarchical(tax) {
        parent_opts(&all_terms, cur_parent, &desc_ids, &mut std::collections::HashSet::new(), &mut parent_options);
    }
    let parent_field = if hierarchical(tax) {
        format!(
            r##"<div class="form-field term-parent-wrap">
<label for="parent">Parent {noun}</label>
<select name="parent" id="parent">{parent_options}</select>
<p>{noun}s can have a hierarchy. Totally optional.</p>
</div>"##
        )
    } else {
        String::new()
    };

    // Tag cloud (non-hierarchical only, WP renders it on the add-form).
    let mut cloud = String::new();
    if !hierarchical(tax) && edit.is_none() {
        let tops = crud::top_terms(db, tax).unwrap_or_default();
        if !tops.is_empty() {
            let max = tops.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1) as f64;
            let mut links = String::new();
            for (i, (name, n)) in tops.iter().enumerate() {
                let size = 10.0 + 12.0 * (*n as f64) / max;
                links.push_str(&format!(
                    r##"<a href="#" class="tag-cloud-link" style="font-size:{size:.0}pt" title="{n} items">{name}</a>{sep}"##,
                    size = size,
                    n = n,
                    name = esc(name),
                    sep = if i + 1 < tops.len() { " " } else { "" }
                ));
            }
            cloud = format!(
                r##"<p><a id="tag-cloud-link" href="#">Choose from the most used {title}</a></p>
<div id="tagcloud" class="the-tagcloud" style="display:none">{links}</div>"##,
                title = title.to_lowercase(),
                links = links,
            );
        }
    }

    // Screen Options panel + toggle tab — persisted per user per taxonomy.
    let chk = |c: &str| if opts.hide.iter().any(|h| h == c) { "" } else { " checked" };
    let screen_meta = format!(
        r##"<div id="screen-meta" class="metabox-prefs" style="display:none">
<div id="screen-options-wrap" class="hidden" tabindex="-1" style="display:block">
<form method="get" action="/admin/tax/{tax}" class="adv-settings">
<input type="hidden" name="screen_options" value="1">
<input type="hidden" name="s" value="{search}">
<fieldset class="metabox-prefs"><legend>Columns</legend>
<label><input class="hide-column-tog" name="hide_description" type="checkbox" value="1"{d}>Description</label>
<label><input class="hide-column-tog" name="hide_slug" type="checkbox" value="1"{g}>Slug</label>
<label><input class="hide-column-tog" name="hide_count" type="checkbox" value="1"{c}>Count</label>
</fieldset>
<fieldset class="screen-options"><legend>Pagination</legend>
<label for="edit_tags_per_page">Number of items per page:</label>
<input type="number" step="1" min="1" max="999" class="screen-per-page" name="per_page" id="edit_tags_per_page" value="{pp}">
</fieldset>
<p class="submit"><input type="submit" class="button button-primary" value="Apply"></p>
</form>
</div>
</div>
<div id="screen-meta-links">
<div id="screen-options-link-wrap" class="hide-if-no-js screen-meta-toggle">
<button type="button" id="show-settings-link" class="button show-settings">Screen Options<span class="toggle-indicator" aria-hidden="true"></span></button>
</div>
</div>"##,
        tax = tax,
        search = search_val,
        d = chk("description"),
        g = chk("slug"),
        c = chk("count"),
        pp = per_page,
    );

    let (edit_name, edit_slug, edit_desc, edit_id, add_label) = if let Some(e) = edit {
        (esc(&e.name), esc(&e.slug), esc(&e.description), format!(r#"<input type="hidden" name="id" value="{}">"#, e.id), format!("Update {noun}"))
    } else {
        (String::new(), String::new(), String::new(), String::new(), format!("Add {noun}"))
    };
    let h1_suffix = if edit.is_some() { " Edit Item" } else { "" };
    let pager = term_pager(&format!("/admin/tax/{tax}"), search_q, page, per_page, total);
    format!(
        r##"{chrome}<div class="wrap nosubsub">
{screen_meta}
<h1 class="wp-heading-inline">{title}{h1_suffix}</h1>
<div id="ajax-response"></div>
<div id="col-container" class="wp-clearfix">
<div id="col-left"><div class="col-wrap">
<div class="form-wrap">
<h2>Add {noun}</h2>
<form method="post" action="/admin/tax/save" class="validate">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="tax" value="{tax}">
{edit_id}
<div class="form-field form-required term-name-wrap">
<label for="tag-name">Name</label>
<input name="name" id="tag-name" type="text" value="{edit_name}" size="40" required>
<p>The name is how it appears on your site.</p>
</div>
<div class="form-field term-slug-wrap">
<label for="tag-slug">Slug</label>
<input name="slug" id="tag-slug" type="text" value="{edit_slug}" size="40">
<p>The “slug” is the URL-friendly version of the name. It is usually all lowercase and contains only letters, numbers, and hyphens.</p>
</div>
{parent_field}
<div class="form-field term-description-wrap">
<label for="tag-description">Description</label>
<textarea name="description" id="tag-description" rows="5" cols="40">{edit_desc}</textarea>
<p>The description is not prominent by default; however, some themes may show it.</p>
</div>
{cloud}
<p class="submit"><input type="submit" name="submit" id="submit" class="button button-primary" value="{add_label}"></p>
</form>
</div>
</div></div>
<div id="col-right"><div class="col-wrap">
<form method="get" action="/admin/tax/{tax}">
<p class="search-box">
<label class="screen-reader-text" for="tag-search-input">Search {title}:</label>
<input type="search" id="tag-search-input" name="s" value="{search}">
<input type="submit" id="search-submit" class="button" value="Search {title}">
</p>
</form>
<form method="post" action="/admin/tax/bulk">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="tax" value="{tax}">
<div class="tablenav top">
<div class="alignleft actions bulkactions">
<select name="action" id="bulk-action-selector-top"><option value="-1">Bulk actions</option><option value="delete">Delete</option></select>
<input type="submit" id="doaction" class="button action" value="Apply">
</div>
{pager}
</div>
<table class="wp-list-table widefat fixed striped tags">
<thead><tr>
<td class="manage-column column-cb check-column"><input type="checkbox"></td>
<th scope="col" class="manage-column column-name column-primary sortable asc"><a>Name</a></th>
<th scope="col" class="manage-column column-description sortable{desc_col}"><a>Description</a></th>
<th scope="col" class="manage-column column-slug sortable{slug_col}"><a>Slug</a></th>
<th scope="col" class="manage-column column-posts sortable{count_col}"><a>Count</a></th>
</tr></thead>
<tbody id="the-list"><tr id="qe-terms" class="inline-edit-row quick-edit-row" style="display:none"><td colspan="5" class="colspanchange">
<input form="qe-terms-form" type="hidden" name="csrf" value="{csrf}">
<input form="qe-terms-form" type="hidden" name="tax" value="{tax}">
<input form="qe-terms-form" type="hidden" name="id" value="">
<fieldset><legend class="inline-edit-legend">Quick Edit</legend>
<label class="inline-edit-group" style="display:block;margin:6px 0"><span class="title" style="display:inline-block;width:90px">Name</span><input form="qe-terms-form" type="text" name="name" id="qe-t-name" style="width:60%"></label>
<label class="inline-edit-group" style="display:block;margin:6px 0"><span class="title" style="display:inline-block;width:90px">Slug</span><input form="qe-terms-form" type="text" name="slug" id="qe-t-slug" style="width:60%"></label>
<label class="inline-edit-group" style="display:block;margin:6px 0"><span class="title" style="display:inline-block;width:90px">Language</span><select form="qe-terms-form" name="lang" id="qe-t-lang"><option value="">—</option><option value="fr">Français</option><option value="es">Español</option></select></label>
<p class="submit inline-edit-save" style="margin-top:8px">
<button form="qe-terms-form" type="submit" class="button button-primary">Update {noun}</button>
<button type="button" class="button" id="qe-t-cancel">Cancel</button>
</p>
</fieldset>
</td></tr>
{rows}</tbody>
<form method="post" action="/admin/tax/save" id="qe-terms-form"></form>
</table>
</form>
<script>
document.addEventListener('click',function(e){{
  var qe=document.getElementById('qe-terms');
  var a=e.target.closest('a.editinline');
  if(a){{
    e.preventDefault();
    var tr=a.closest('tr');
    document.querySelectorAll('#the-list tr').forEach(function(r){{if(r.id!=='qe-terms')r.style.display=''}});
    qe.querySelector('input[name=id]').value=tr.id.replace('term-','');
    qe.querySelector('#qe-t-name').value=tr.getAttribute('data-name')||'';
    qe.querySelector('#qe-t-slug').value=tr.getAttribute('data-slug')||'';
    var slug=tr.getAttribute('data-slug')||'';
    qe.querySelector('#qe-t-lang').value=slug.endsWith('-es')?'es':'fr';
    tr.parentNode.insertBefore(qe,tr.nextSibling);
    qe.style.display='';
    tr.style.display='none';
    qe.querySelector('#qe-t-name').focus();
    return;
  }}
  if(e.target&&e.target.id==='qe-t-cancel'){{
    qe.style.display='none';
    document.querySelectorAll('#the-list tr').forEach(function(r){{if(r.id!=='qe-terms')r.style.display=''}});
  }}
  if(e.target&&e.target.id==='show-settings-link'){{
    var sm=document.getElementById('screen-meta');
    if(sm)sm.style.display=sm.style.display==='none'?'':'none';
  }}
  if(e.target&&e.target.id==='tag-cloud-link'){{
    e.preventDefault();
    var tc=document.getElementById('tagcloud');
    if(tc)tc.style.display=tc.style.display==='none'?'':'none';
  }}
  var cl=e.target.closest('a.tag-cloud-link');
  if(cl){{
    e.preventDefault();
    var ti=document.getElementById('tag-name');
    if(ti){{ti.value=cl.textContent;ti.focus();}}
  }}
}});
</script>
</div></div>
</div></div>"##,
        chrome = chrome(tax_menu(tax), s),
        csrf = s.csrf,
        search = search_val,
        screen_meta = screen_meta,
        parent_field = parent_field,
        cloud = cloud,
        pager = pager,
        desc_col = col("description"),
        slug_col = col("slug"),
        count_col = col("count"),
    )
}
/// Menu section a taxonomy belongs under: post taxonomies → Posts,
/// everything else → Tour (tourmaster fields + ES variants live there).
fn tax_menu(tax: &str) -> &'static str {
    match tax {
        "category" | "post_tag" => "posts",
        _ => "tours",
    }
}

pub(crate) async fn tax_page(State(app): State<App>, headers: HeaderMap, Path(tax): Path<String>, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    // "Add New Filter" (tourmaster custom taxonomy creator) lives under /tax/.
    if tax == "new-filter" {
        return html_admin(&s, "Add New Filter", &new_filter_html(&s), true, "edit-tags").into_response();
    }
    if !crud::taxonomy_exists(&sc.db, &tax).unwrap_or(false)
        && !custom_taxs(&sc.db).contains_key(tax.as_str())
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let search = q.get("s").cloned().unwrap_or_default();
    // Screen Options save (GET form — no dedicated POST route exists):
    // `screen_options=1` marks the submit so absent checkboxes mean hidden.
    if q.get("screen_options").is_some() {
        let mut hide = Vec::new();
        for (param, col) in [
            ("hide_description", "description"),
            ("hide_slug", "slug"),
            ("hide_count", "count"),
        ] {
            if q.get(param).is_none() {
                hide.push(col.to_string());
            }
        }
        let per_page = q
            .get("per_page")
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(20)
            .clamp(1, 999);
        let _ = crud::save_screen_opts(
            &sc.db,
            s.user_id,
            &tax,
            &crud::TaxScreenOpts { hide, per_page },
        );
        let back = if search.is_empty() {
            format!("/admin/tax/{tax}")
        } else {
            format!("/admin/tax/{tax}?s={}", urlenc(&search))
        };
        return Redirect::to(&back).into_response();
    }
    let page: i64 = q.get("paged").and_then(|v| v.parse().ok()).unwrap_or(1);
    html_admin(&s, "Terms", &tax_page_html(&sc.db, &s, &tax, None, &search, page), true, "edit-tags").into_response()
}


pub(crate) async fn tax_edit(State(app): State<App>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    match crud::get_term(&sc.db, id) {
        Ok(Some(t)) => {
            let tax = crud::taxonomy_of(&sc.db, id).unwrap_or(None).unwrap_or_else(|| "category".into());
            html_admin(&s, "Edit", &tax_page_html(&sc.db, &s, &tax, Some(&t), "", 1), true, "edit-tags").into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) async fn tax_save(State(app): State<App>, headers: HeaderMap, Form(f): Form<TaxForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let slug = if f.slug.trim().is_empty() {
        crud::slugify(&f.name)
    } else {
        f.slug.clone()
    };
    match crud::save_term(&sc.db, &f.tax, f.id, &f.name, &slug, &f.description, f.parent) {
        Ok(_) => {
            crate::admin::admin::bust_cache(&sc);
            Redirect::to(&format!("/admin/tax/{}", f.tax)).into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub(crate) async fn tax_delete(State(app): State<App>, headers: HeaderMap, Form(f): Form<TaxDelete>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let _ = crud::delete_term(&sc.db, f.id);
    crate::admin::admin::bust_cache(&sc);
    Redirect::to(&format!("/admin/tax/{}", f.tax)).into_response()
}

pub(crate) async fn tax_bulk(State(app): State<App>, headers: HeaderMap, Form(map): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if map.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let tax = map.get("tax").cloned().unwrap_or_else(|| "category".into());
    if map.get("action").map(|a| a.as_str()) == Some("delete") {
        for k in map.keys() {
            if let Some(id) = k.strip_prefix("d-").and_then(|x| x.parse::<i64>().ok()) {
                let _ = crud::delete_term(&sc.db, id);
            }
        }
        crate::admin::admin::bust_cache(&sc);
    }
    Redirect::to(&format!("/admin/tax/{tax}")).into_response()
}




