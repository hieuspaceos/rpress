#[allow(unused_imports)]
use crate::sql;
use super::*;

/// Content-type registry: kind/section → `plugins::KindDef`.
/// Adding a CPT = one `KindDef` on a `PluginAdapter`; core `post`/`page`
/// are registry built-ins. Unknown kinds are `None` — callers 404.
pub(crate) fn spec_for_kind(kind: &str) -> Option<&'static plugins::KindDef> {
    plugins::kind_def(kind)
}

pub(crate) fn spec_for_section(section: &str) -> Option<&'static plugins::KindDef> {
    plugins::kind_for_section(section)
}

pub(crate) fn kind_for(section: &str) -> Option<&'static plugins::KindDef> {
    plugins::kind_for_section(section)
}

pub(crate) async fn section(
    State(app): State<App>,
    headers: HeaderMap,
    Path(section): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let lang = q.get("lang").map(|s| s.as_str()).unwrap_or("");
    if matches!(section.as_str(), "media" | "redirects" | "publish") {
        let inner = format!(
            "{}<div class=\"wrap\"><h1>{}</h1><p class=muted>Use the dedicated screen in the menu.</p></div>",
            chrome(&section, &s),
            section
        );
        return html_page(&s, &section, &inner, true).into_response();
    }
    let Some(spec) = kind_for(&section) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let kind = spec.kind;
    let status = q.get("status").map(|s| s.as_str()).unwrap_or("");
    let tax = q.get("tax").map(|s| s.as_str()).unwrap_or("");
    let cat = q.get("cat").map(|s| s.as_str()).unwrap_or("");
    // Taxonomy count links (/admin/tax/*) pass a terms.id; resolved to
    // taxonomy+slug inside list_admin.
    let term: i64 = q.get("term").and_then(|s| s.parse().ok()).unwrap_or(0);
    let search = q.get("s").map(|s| s.as_str()).unwrap_or("");
    let page: i64 = q.get("paged").and_then(|s| s.parse().ok()).unwrap_or(1);
    let per_page = 20i64;
    let (rows, total, counts) = match crud::list_admin(&sc.db, kind, status, lang, tax, cat, term, search, page, per_page)
    {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let inner = render_list(
        sc,
        &section, kind, &s, &rows, total, &counts, status, lang, tax, cat, search, page, per_page,
    );
    html_admin(&s,
        match kind_for(&section) {
            Some(k) => k.list_title,
            None => "List",
        },
        &inner,
        true,
        "edit-php",
    )
    .into_response()
}

/// Absolute preview URL on the *public* site (settings.site_url),
/// e.g. https://test.example.com/voyage/slug/ — never admin-host relative.
pub(crate) fn public_path(sc: &SiteCtx, lang: &str, kind: &str, slug: &str) -> String {
    let base = site_url(sc).trim_end_matches('/').to_string();
    // LangConfig is settings-derived; a failed conn falls back to the
    // primary-only config (empty prefix) rather than panicking on a
    // preview-link render.
    let cfg = crate::db::conn(&sc.db)
        .map(|c| crate::lang::config(&c))
        .unwrap_or_default();
    let prefix = crate::lang::url_prefix(&cfg, lang);
    if let Some(b) = plugins::base_for(&sc.kind_bases, kind) {
        format!("{base}{prefix}/{b}/{slug}/")
    } else {
        format!("{base}{prefix}/{slug}/")
    }
}

pub(crate) fn dash(s: &str) -> String {
    if s.is_empty() {
        "—".into()
    } else {
        esc(s)
    }
}

pub(crate) fn pll_cell(r: &crud::AdminListRow, kind: &str, langs: &crate::lang::LangConfig) -> String {
    let flag = |code: &str, id: Option<i64>| match id {
        Some(n) => format!(
            r#"<span class="pll-column-flag"><a href="/admin/edit/{n}">{code}</a></span>"#
        ),
        None => format!(
            r#"<a class="pll_icon_add" href="/admin/new/{kind}?from={}&amp;lang={code}">+</a>"#,
            r.id
        ),
    };
    let mut out = String::new();
    for code in &langs.langs {
        let id = if r.lang == *code {
            Some(r.id)
        } else {
            r.trans_map.get(code).copied()
        };
        out.push_str(&flag(&code.to_uppercase(), id));
        out.push(' ');
    }
    out.trim_end().to_string()
}

pub(crate) fn qs(status: &str, lang: &str, tax: &str, page: i64) -> String {
    // Every interpolated component is urlenc'd — these land verbatim inside
    // href="{}" attributes; a raw quote in ?tax= is reflected XSS.
    let mut p = Vec::new();
    if !status.is_empty() {
        p.push(format!("status={}", super::urlenc(status)));
    }
    if !lang.is_empty() {
        p.push(format!("lang={}", super::urlenc(lang)));
    }
    if !tax.is_empty() {
        p.push(format!("tax={}", super::urlenc(tax)));
    }
    if page > 1 {
        p.push(format!("paged={page}"));
    }
    if p.is_empty() {
        "?".into()
    } else {
        format!("?{}", p.join("&"))
    }
}

pub(crate) fn render_list(
    sc: &SiteCtx,
    section: &str,
    kind: &str,
    s: &Sess,
    rows: &[crud::AdminListRow],
    total: i64,
    counts: &crud::ListCounts,
    status: &str,
    lang: &str,
    tax: &str,
    cat: &str,
    search: &str,
    page: i64,
    per_page: i64,
) -> String {
    let (h1, add, add_kind) = match kind_for(section) {
        Some(k) => (k.list_title, k.add_label, k.kind),
        None => (section, "Add New", kind),
    };
    let on = |st: &str, lg: &str| {
        if status == st && lang == lg {
            " class=\"current\""
        } else {
            ""
        }
    };
    // N-language subnav: one filter per configured language (phase 4 —
    // the hardwired fr/es pair retired with post_translations).
    let lang_cfg = crate::db::conn(&sc.db)
        .map(|c| crate::lang::config(&c))
        .unwrap_or_default();
    let mut lang_links = String::new();
    for (i, lg) in lang_cfg.langs.iter().enumerate() {
        let sep = if i + 1 < lang_cfg.langs.len() { " |" } else { "" };
        lang_links.push_str(&format!(
            r#"<li><a href="{}"{}>{} <span class="count">({})</span></a>{sep}</li>"#,
            qs("", lg, tax, 1),
            on("", lg),
            crate::lang::lang_label(lg),
            counts.lang_counts.get(lg).copied().unwrap_or(0),
        ));
        lang_links.push('\n');
    }
    let sub = format!(
        r#"<ul class="subsubsub">
<li class="all"><a href="{}"{}>All <span class="count">({})</span></a> |</li>
<li class="publish"><a href="{}"{}>Published <span class="count">({})</span></a> |</li>
<li class="pending"><a href="{}"{}>Pending <span class="count">({})</span></a> |</li>
<li class="future"><a href="{}"{}>Scheduled <span class="count">({})</span></a> |</li>
<li class="draft"><a href="{}"{}>Drafts <span class="count">({})</span></a> |</li>
<li class="trash"><a href="{}"{}>Trash <span class="count">({})</span></a> |</li>
{lang_links}</ul>"#,
        qs("", "", tax, 1),
        on("", ""),
        counts.all,
        qs("publish", "", tax, 1),
        on("publish", ""),
        counts.publish,
        qs("pending", "", tax, 1),
        on("pending", ""),
        counts.pending,
        qs("future", "", tax, 1),
        on("future", ""),
        counts.future,
        qs("draft", "", tax, 1),
        on("draft", ""),
        counts.draft,
        qs("trash", "", tax, 1),
        on("trash", ""),
        counts.trash,
        lang_links = lang_links,
    );
    // Shared pager — carries every active filter so paging doesn't drop
    // status/lang/tax/cat/search context.
    let pager = {
        let mut extra = HashMap::new();
        for (k, v) in [("status", status), ("lang", lang), ("tax", tax), ("cat", cat), ("s", search)] {
            if !v.is_empty() {
                extra.insert(k.to_string(), v.to_string());
            }
        }
        list::Pager::at(page, total, per_page).nav("", &extra)
    };
    let head = spec_for_section(section).map(|k| k.head).unwrap_or("");
    let mut body = String::new();
    for r in rows {
        let title = if r.title.is_empty() {
            "(no title)".into()
        } else {
            esc(&r.title)
        };
        let preview = if r.status == "publish" {
            public_path(sc, &r.lang, kind, &r.slug)
        } else {
            // Non-published rows preview through the session-gated renderer
            // (6.1) — the public URL would 404 for drafts/future.
            format!("/admin/posts/{}/preview", r.id)
        };
        let view_label = if r.status == "publish" && plugins::kind_def(kind).map(|d| d.url_base_key.is_some()).unwrap_or(false) { "View" } else { "Preview" };
        let (next_status, act) = if r.status == "trash" {
            ("draft", "Restore")
        } else {
            ("trash", "Trash")
        };
        let actions = format!(
            r##"<div class="row-actions"><span class="edit"><a href="/admin/edit/{0}">Edit</a> | </span><span class="inline hide-if-no-js"><a href="#{0}" class="editinline" data-id="{0}">Quick Edit</a> | </span><span class="trash"><form method="post" action="/admin/trash" style="display:inline"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{0}"><input type="hidden" name="status" value="{next_status}"><button class="button-link">{act}</button></form> | </span><span class="view"><a href="{preview}" target="_blank">{view_label}</a> | </span><span class="clone"><form method="post" action="/admin/clone" style="display:inline"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{0}"><button class="button-link">Clone</button></form> | </span><span class="new-draft"><form method="post" action="/admin/clone" style="display:inline"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{0}"><input type="hidden" name="draft" value="1"><button class="button-link">New Draft</button></form></span></div>"##,
            r.id,
            csrf = s.csrf,
            preview = esc(&preview),
        );
        let extra = format!(
            "{}{}",
            spec_for_section(section).map(|k| (k.row_cols)(r, kind, &lang_cfg)).unwrap_or_default(),
            // SEO cell is filled lazily by /admin/seo/cell/{id} —
            // running seo_analysis per row here made list pages slow.
            format!(
                r#"<td class="rm-seo-col" data-seo-id="{}"><div class="rm-seo-pill rm-seo-na">…</div></td>"#,
                r.id
            )
        );
        let slug_d = esc(&r.slug);
        let status_d = esc(&r.status);
        // WP post states after the title — "Scheduled" is the only one the
        // status filter can't already tell at a glance.
        let state = if r.status == "future" {
            r#" — <span class="post-state">Scheduled</span>"#
        } else {
            ""
        };
        body.push_str(&format!(
            r#"<tr id="post-{id}" data-slug="{slug_d}" data-status="{status_d}"><th scope="row" class="check-column"><input form="posts-bulk" type="checkbox" name="p-{id}"></th><td class="title column-title column-primary"><strong><a class="row-title" href="/admin/edit/{id}">{title}</a>{state}</strong>{actions}</td>{extra}</tr>"#,
            id = r.id
        ));
    }
    if body.is_empty() {
        body.push_str(&list::empty_row(8, "No items found."));
    }
    // Category filter only applies to posts; pages/tours get just the
    // (cosmetic) All dates select like WP's page list.
    let cat_sel = if kind == "post" {
        let mut o = String::from(r#"<select name="cat" id="cat"><option value="">All Categories</option>"#);
        if let Ok(list) = crud::term_choices(&sc.db, "category", &[]) {
            for (slug, name, _) in list.iter().take(40) {
                let sel = if slug == cat { " selected" } else { "" };
                o.push_str(&format!(r#"<option value="{}"{}>{}</option>"#, esc(slug), sel, esc(name)));
            }
            if list.len() > 40 {
                o.push_str(&format!(r#"<option disabled>… ({} more — type to search)</option>"#, list.len() - 40));
            }
        }
        o.push_str("</select>");
        o
    } else {
        String::new()
    };
    let search_label = match kind {
        "page" => "Pages",
        k => plugins::kind_def(k).map(|d| d.list_title).unwrap_or("Posts"),
    };
    // GET form drops the current query string — carry status/lang/tax
    // forward as hidden inputs so a search or filter doesn't reset them.
    let mut hidden = String::new();
    for (k, v) in [("status", status), ("lang", lang), ("tax", tax)] {
        if !v.is_empty() {
            hidden.push_str(&format!(
                r#"<input type="hidden" name="{}" value="{}">"#,
                k,
                esc_attr(v)
            ));
        }
    }
    // WP parity with the comments Trash view: an Empty Trash button
    // appears in the bulkactions bar only when viewing status=trash.
    // WP shows it for pages too, so all kinds get it.
    let empty_btn = if status == "trash" {
        format!(
            r#"<button type="submit" class="button" form="posts-bulk" formaction="/admin/posts/empty?kind={}">Empty Trash</button>"#,
            super::urlenc(kind)
        )
    } else {
        String::new()
    };
    let filters = format!(
        r##"<form method="get" action="">
{hidden}<div class="tablenav top"><div class="alignleft actions bulkactions">
<label for="bulk-action-selector-top" class="screen-reader-text">Select bulk action</label>
<select form="posts-bulk" name="action" id="bulk-action-selector-top">
<option value="-1">Bulk actions</option>
<option value="edit">Edit</option>
<option value="trash">Move to Trash</option>
</select>
<input form="posts-bulk" type="submit" id="doaction" class="button action" value="Apply">
{empty_btn}</div>
<div class="alignleft actions">
<select name="date" id="filter-by-date"><option value="0">All dates</option></select>
{cat_sel}<input type="submit" name="filter_action" id="post-query-submit" class="button" value="Filter">
</div>
<p class="search-box"><label class="screen-reader-text" for="post-search-input">Search {search_label}:</label>
<input type="search" id="post-search-input" name="s" value="{search_val}">
<input type="submit" id="search-submit" class="button" value="Search {search_label}"></p>
{pager}
</div>
</form>"##,
        cat_sel = cat_sel,
        search_val = esc(search),
        empty_btn = empty_btn,
        hidden = hidden,
        search_label = search_label,
        pager = pager
    );
    let qe = {
        let tax_block = if kind == "post" {
            let mut cat_checks = String::new();
            if let Ok(list) = crud::term_choices(&sc.db, "category", &[]) {
                for (slug, name, _) in list.iter().take(80) {
                    cat_checks.push_str(&format!(
                        r#"<li><label><input type="checkbox" name="cat" value="{}"> {}</label></li>"#,
                        esc(slug), esc(name)
                    ));
                }
                if list.len() > 80 {
                    cat_checks.push_str(&format!(
                        r#"<li><label class="howto">… ({} more — use the filter above)</label></li>"#,
                        list.len() - 80
                    ));
                }
            }
            format!(
                r#"<div>
<span class="title inline-edit-categories-label">Categories</span>
<input type="search" id="qe-cat-search" placeholder="Filter categories…" style="width:100%;max-width:240px;margin-bottom:4px">
<ul id="qe-cats" class="categorychecklist" style="max-height:180px;overflow:auto;margin:4px 0;border:1px solid #ddd;padding:6px;min-width:240px">{cat_checks}</ul>
</div>
<div>
<span class="title">Tags</span>
<textarea form="qe-posts-form" name="tags" id="qe-tags" rows="3" cols="34" placeholder=""></textarea>
<p class="howto" style="margin:0 0 8px">Separate tags with commas</p>
<label class="inline-edit-group"><span class="title">Comments</span><input type="checkbox" name="comments" value="1"> Allow Comments</label>
<label class="inline-edit-group"><span class="title">Sticky</span><input type="checkbox" name="sticky" value="1"> Make this post sticky</label></div>"#
            )
        } else {
            String::new()
        };
        let lang_opts: String = std::iter::once(r#"<option value="">—</option>"#.to_string())
            .chain(lang_cfg.langs.iter().map(|lg| {
                format!(r#"<option value="{}">{}</option>"#, esc(lg), esc(&crate::lang::lang_label(lg)))
            }))
            .collect();
        format!(
            r##"<tr id="qe-row" class="inline-edit-row quick-edit-row" style="display:none"><td colspan="6" class="colspanchange">
<input form="qe-posts-form" type="hidden" name="csrf" value="{csrf}">
<input form="qe-posts-form" type="hidden" name="id" value="">
<input form="qe-posts-form" type="hidden" name="cats" value="">
<fieldset><legend class="inline-edit-legend">Quick Edit</legend>
<div class="inline-edit-col" style="display:flex;gap:30px;flex-wrap:wrap">
<div>
<label class="inline-edit-group"><span class="title">Title</span><input form="qe-posts-form" type="text" name="title" id="qe-title" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">Slug</span><input form="qe-posts-form" type="text" name="slug" id="qe-slug" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">Status</span>
<select form="qe-posts-form" name="status" id="qe-status"><option value="publish">Published</option><option value="future">Scheduled</option><option value="draft">Draft</option><option value="pending">Pending Review</option><option value="trash">Trash</option></select></label>
<label class="inline-edit-group"><span class="title">Language</span><select form="qe-posts-form" name="lang" id="qe-lang">{lang_opts}</select></label>
</div>
{tax_block}
</div>
<p class="submit inline-edit-save" style="margin-top:10px">
<button form="qe-posts-form" type="submit" class="button button-primary">Update</button>
<button type="button" class="button" id="qe-cancel">Cancel</button>
</p>
</fieldset>
</td></tr>"##,
            csrf = s.csrf,
            lang_opts = lang_opts,
            tax_block = tax_block
        )
    };
    let be = if kind == "post" {
        let mut cat_checks = String::new();
        if let Ok(list) = crud::term_choices(&sc.db, "category", &[]) {
            for (slug, name, _) in list.iter().take(80) {
                cat_checks.push_str(&format!(
                    r#"<li><label><input type="checkbox" name="bcat-{}" value="1"> {}</label></li>"#,
                    esc(slug), esc(name)
                ));
            }
            if list.len() > 80 {
                cat_checks.push_str(&format!(
                    r#"<li><label class="howto">… ({} more)</label></li>"#,
                    list.len() - 80
                ));
            }
        }
        format!(
            r##"<tr id="be-row" class="inline-edit-row bulk-edit-row" style="display:none"><td colspan="6" class="colspanchange">
<input form="posts-bulk" type="hidden" name="ids" id="be-ids" value="">
<fieldset><legend class="inline-edit-legend">Bulk Edit</legend>
<div style="display:flex;gap:30px;flex-wrap:wrap">
<div><span class="title inline-edit-categories-label">Categories</span>
<ul id="be-cats" class="categorychecklist" style="max-height:160px;overflow:auto;margin:4px 0;border:1px solid #ddd;padding:6px;min-width:220px">{cat_checks}</ul></div>
<div>
<span class="title">Tags</span>
<textarea form="posts-bulk" name="btags" id="be-tags" rows="3" cols="34"></textarea>
<p class="howto" style="margin:0 0 8px">Separate tags with commas</p>
<label class="inline-edit-group"><span class="title">Status</span>
<select form="posts-bulk" name="bstatus" id="be-status"><option value="">— No Change —</option><option value="publish">Published</option><option value="future">Scheduled</option><option value="draft">Draft</option><option value="pending">Pending Review</option><option value="trash">Trash</option></select></label>
</div>
</div>
<p class="submit inline-edit-save" style="margin-top:10px">
<button form="posts-bulk" type="submit" class="button button-primary" name="be_go" value="1">Update</button>
<button type="button" class="button" id="be-cancel">Cancel</button>
</p>
</fieldset>
</td></tr>"##,
            cat_checks = cat_checks
        )
    } else {
        String::new()
    };
    format!(
        r#"<style>
.rm-seo-col{{width:190px}}
.rm-seo-pill{{display:inline-block;padding:3px 10px;border-radius:3px;color:#fff;font-weight:600;font-size:12px}}
.rm-seo-good{{background:#46b450}}.rm-seo-ok{{background:#ffb900}}.rm-seo-bad{{background:#dc3232}}.rm-seo-na{{background:#787c82}}
.rm-seo-kw{{margin:6px 0 2px;font-size:13px}}
.rm-seo-links{{color:#787c82;font-size:12px}}
</style>{}<div class="wrap"><h1 class="wp-heading-inline">{h1}</h1>
<a href="/admin/new/{add_kind}" class="page-title-action">{add}</a>
<hr class="wp-header-end">
{sub}{filters}
<form method="post" action="/admin/bulk" id="posts-bulk"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}">
<table class="wp-list-table widefat fixed striped table-view-list posts"><thead>{head}</thead><tbody>{body}{qe}{be}</tbody></table>
</form>
<form method="post" action="/admin/quickedit" id="qe-posts-form"></form>
</div>
<script>
(function(){{
var qe=document.getElementById('qe-row');if(!qe)return;
var origTr=null;
  // live filter inside the checklist (respects lang-scope via dataset.langHidden)
  var cs = qe.querySelector('#qe-cat-search');
  if (cs) {{
    cs.addEventListener('input', function() {{
      var q = cs.value.toLowerCase();
      qe.querySelectorAll('#qe-cats li').forEach(function(li) {{
        if (li.dataset.langHidden === '1') return;
        li.style.display = li.textContent.toLowerCase().includes(q) ? '' : 'none';
      }});
    }});
  }}
document.addEventListener('click',function(e){{
  var a=e.target.closest('a.editinline');
  if(a){{
    e.preventDefault();
    if(origTr)origTr.style.display='';
    var tr=a.closest('tr');origTr=tr;var id=a.getAttribute('data-id');
    qe.querySelector('input[name=id]').value=id;
    qe.querySelector('#qe-title').value=tr.querySelector('.row-title').textContent;
    qe.querySelector('#qe-slug').value=tr.getAttribute('data-slug')||'';
    qe.querySelector('#qe-status').value=tr.getAttribute('data-status')||'publish';
    qe.querySelectorAll('#qe-cats input').forEach(function(c){{c.checked=false}});
    var tagsEl = qe.querySelector('#qe-tags');
    if(tagsEl)tagsEl.value='';
    fetch('/admin/postdata/'+id).then(function(r){{return r.json()}}).then(function(d){{
      var allowed = d.allowed_cats ? new Set(d.allowed_cats) : null;
      if(allowed){{
        qe.querySelectorAll('#qe-cats li').forEach(function(li){{
          var inp = li.querySelector('input');
          var hide = !(inp && allowed.has(inp.value));
          li.dataset.langHidden = hide ? '1' : '';
          li.style.display = hide ? 'none' : '';
        }});
      }}
      (d.cats||[]).forEach(function(s){{var c=qe.querySelector('#qe-cats input[value="'+s+'"]');if(c)c.checked=true}});
      if(tagsEl)tagsEl.value=(d.tags||[]).join(', ');
      if(d.lang)qe.querySelector('#qe-lang').value=d.lang;
    }});
    tr.parentNode.insertBefore(qe,tr.nextSibling);
    qe.style.display='';
    return;
  }}
  if(e.target&&e.target.id==='qe-cancel'){{
    qe.style.display='none';
    if(origTr)origTr.style.display='';
  }}
  if(e.target&&e.target.type==='submit'&&e.target.closest('#qe-row')){{
    var ci = qe.querySelectorAll('#qe-cats input:checked');
    qe.querySelector('input[name=cats]').value=[...ci].map(c=>c.value).join(',');
  }}
  // bulk apply
  if(e.target&&e.target.id==='doaction'){{
    var sel=document.getElementById('bulk-action-selector-top');
    var checked=[...document.querySelectorAll('#the-list input[type=checkbox]:checked, .wp-list-table input[name^=p-]:checked')];
    var ids=checked.map(function(c){{return c.name.replace('p-','')}});
    if(sel&&sel.value==='edit'&&ids.length){{
      var be=document.getElementById('be-row');
      if(!be)return;
      e.preventDefault();
      be.querySelector('#be-ids').value=ids.join(',');
      document.getElementById('bulk-action-selector-top').value='-1';
      var first=checked[0].closest('tr');
      first.parentNode.insertBefore(be,first.nextSibling);
      be.style.display='';
    }}
  }}
  if(e.target&&e.target.id==='be-cancel'){{
    document.getElementById('be-row').style.display='none';
  }}
  if(e.target&&e.target.name==='be_go'){{
    document.getElementById('bulk-action-selector-top').value='edit';
  }}
}});
}})();
</script>
<script>
// Lazy SEO column: one fetch per row fills the placeholder cell with
// the fragment from /admin/seo/cell/{{id}}; failures keep the … pill.
(function(){{
document.querySelectorAll('[data-seo-id]').forEach(function(td){{
  var id=td.getAttribute('data-seo-id');
  fetch('/admin/seo/cell/'+id).then(function(r){{return r.ok?r.text():''}}).then(function(h){{if(h)td.innerHTML=h}}).catch(function(){{}});
}});
}})();
</script>"#,
        chrome(section, s),
        csrf = s.csrf,
        back = format!("/admin/{section}")
    )
}

#[derive(Deserialize)]
pub(crate) struct TrashForm {
    csrf: String,
    id: i64,
    status: String,
}

pub(crate) async fn trash_post(State(app): State<App>, headers: HeaderMap, Form(f): Form<TrashForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let _ = crud::set_post_status(&sc.db, f.id, &f.status);
    // Status flips both ways matter: trash removes the file, restore rewrites it.
    admin::regen_posts(&sc, &[f.id]);
    Redirect::to("/admin/posts").into_response()
}

pub(crate) async fn bulk_posts(State(app): State<App>, headers: HeaderMap, Form(map): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if !ct_eq(s.csrf.as_bytes(), map.get("csrf").map(|x| x.as_str()).unwrap_or("").as_bytes()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut ids: Vec<i64> = Vec::new();
    if let Some(csv) = map.get("ids") {
        ids.extend(csv.split(',').filter_map(|x| x.trim().parse::<i64>().ok()));
    }
    for k in map.keys() {
        if let Some(id) = k.strip_prefix("p-").and_then(|x| x.parse::<i64>().ok()) {
            ids.push(id);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    // Bulk loops are the heaviest admin writes (status flips + term
    // assignments × N posts) — keep them off the tokio worker (H14).
    // regen/bust calls inside already manage their own blocking work.
    let action = map.get("action").cloned();
    {
        let sc = sc.clone();
        let ids = ids.clone();
        let (bstatus, bcats, btags) = (
            map.get("bstatus").cloned().unwrap_or_default(),
            map.keys().filter_map(|k| k.strip_prefix("bcat-").map(|x| x.to_string())).collect::<Vec<String>>(),
            map.get("btags")
                .map(|t| t.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect::<Vec<String>>())
                .unwrap_or_default(),
        );
        let _ = tokio::task::spawn_blocking(move || {
            match action.as_deref() {
                Some("trash") => {
                    for id in &ids {
                        let _ = crud::set_post_status(&sc.db, *id, "trash");
                    }
                    admin::regen_posts(&sc, &ids);
                }
                Some("edit") => {
                    for id in &ids {
                        let _ = crud::bulk_apply(&sc.db, *id, &bstatus, &bcats, &btags);
                    }
                    admin::regen_posts(&sc, &ids);
                    admin::bust_cache(&sc); // taxonomy assignments hit archive pages too
                }
                _ => {}
            }
        })
        .await;
    }
    // Bulk bar is shared by posts/pages/tours — return to the list it ran on.
    let back = map
        .get("back")
        .filter(|b| b.starts_with("/admin/"))
        .map(|b| b.as_str())
        .unwrap_or("/admin/posts");
    Redirect::to(back).into_response()
}

/// Doomed-artifact snapshot taken BEFORE a save commits — both mappings
/// a post-commit cleanup needs:
///   - `terms`  the pre-save (taxonomy, slug) set — regen can't see the
///     replaced assignment afterwards, so archives the post just LEFT
///     would stay stale without it;
///   - `files`  the old URL's variant set — `post_out_rel` needs the
///     pre-commit slug/lang/kind row.
/// `slug/lang/kind` ride along so callers only remove files when the
/// permalink actually moved.
struct PreSave {
    slug: String,
    lang: String,
    kind: String,
    terms: Vec<(String, String)>,
    files: Vec<PathBuf>,
}

fn snapshot_before_save(sc: &SiteCtx, id: i64) -> Option<PreSave> {
    let conn = crate::db::conn(&sc.db).ok()?;
    let (slug, lang, kind): (String, String, String) = conn
        .query_row(sql!("posts__select_slug_lang_kind"), [id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .ok()?;
    let terms = crud::terms_for(&sc.db, id)
        .unwrap_or_default()
        .into_iter()
        .map(|(t, s, _)| (t, s))
        .collect();
    let files = generate::file_variants_for(&conn, id, crate::admin::tour_base(sc)).unwrap_or_default();
    Some(PreSave { slug, lang, kind, terms, files })
}

/// `POST /admin/posts/empty?kind=post` — Empty Trash on the Trash view
/// (mirrors `comments_empty` at comments.rs). Hard-deletes every trashed
/// post of `kind`; dependent terms/meta/tours/comments go with it.
/// Ordering (audit, phase 5): snapshot every doomed file path FIRST —
/// `post_out_rel` needs the DB row — then the transactional delete, then
/// file removal with real error propagation + one retry; if files still
/// remain, bust the cache dir rather than leave stale pages serving.
pub(crate) async fn posts_empty(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if !ct_eq(s.csrf.as_bytes(), f.get("csrf").map(|x| x.as_str()).unwrap_or("").as_bytes()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let kind = q
        .get("kind")
        .or_else(|| f.get("kind"))
        .map(|s| s.as_str())
        .unwrap_or("post");
    let Some(spec) = spec_for_kind(kind) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let kind = spec.kind;
    // 1. Snapshot doomed id → file-variant paths while the rows exist.
    // A snapshot failure means files would be orphaned silently — fail
    // loudly and skip the delete rather than strand stale pages.
    let mut doomed: Vec<PathBuf> = Vec::new();
    let ids = match crud::ids_matching(&sc.db, sql!("posts__select_id_kind_trash"), [kind]) {
        Ok(ids) => ids,
        Err(e) => {
            eprintln!("empty_trash: list doomed ids: {e:#}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match crate::db::conn(&sc.db) {
        Ok(conn) => {
            for id in ids {
                match generate::file_variants_for(&conn, id, crate::admin::tour_base(sc)) {
                    Ok(v) => doomed.extend(v.into_iter().map(|r| sc.out.join(r))),
                    Err(e) => eprintln!("empty_trash: resolve post {id}: {e:#}"),
                }
            }
        }
        Err(e) => {
            eprintln!("empty_trash: snapshot conn: {e:#}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    // 2. The hard delete — propagate the error, don't swallow it.
    if let Err(e) = crud::empty_trash(&sc.db, kind) {
        eprintln!("empty_trash: {e:#}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    // 3. Remove the files; retry failures once, then bust the cache dir —
    // a served stale file is worse than a cold cache.
    doomed.sort();
    doomed.dedup();
    let mut remaining: Vec<PathBuf> = Vec::new();
    for p in &doomed {
        if p.is_file() {
            if let Err(e) = std::fs::remove_file(p) {
                eprintln!("empty_trash: remove {}: {e}", p.display());
                remaining.push(p.clone());
            }
        }
    }
    for p in std::mem::take(&mut remaining) {
        if p.is_file() && std::fs::remove_file(&p).is_err() {
            remaining.push(p);
        }
    }
    if !remaining.is_empty() {
        eprintln!("empty_trash: {} file(s) survived removal — busting cache", remaining.len());
        admin::bust_cache(&sc);
    }
    Redirect::to(&format!("/admin/{}?status=trash", spec.section)).into_response()
}

#[derive(Deserialize)]
pub(crate) struct CloneForm {
    csrf: String,
    id: i64,
    #[serde(default)]
    draft: String,
}

#[derive(Deserialize)]
pub(crate) struct QuickEditForm {
    csrf: String,
    id: i64,
    title: String,
    slug: String,
    status: String,
    #[serde(default)]
    cats: String,
    #[serde(default)]
    tags: String,
    #[serde(default)]
    lang: String,
}
pub(crate) async fn quick_edit(State(app): State<App>, headers: HeaderMap, Form(f): Form<QuickEditForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let cats: Vec<String> = f.cats.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
    let tags: Vec<String> = f.tags.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
    let status = match f.status.as_str() {
        "publish" | "future" | "draft" | "pending" | "trash" => f.status.as_str(),
        _ => "draft",
    };
    // N-language whitelist: the submitted lang must be one of the site's
    // configured languages — anything else falls back to unset.
    let lang_cfg = crate::db::conn(&sc.db).map(|c| crate::lang::config(&c)).unwrap_or_default();
    let lang = if lang_cfg.langs.iter().any(|l| l == &f.lang) { f.lang.as_str() } else { "" };
    // Snapshot old path + term set BEFORE commit (see PreSave).
    let pre = snapshot_before_save(&sc, f.id);
    match crud::quick_update(&sc.db, f.id, &f.title, &f.slug, status, lang, &cats, &tags) {
        Ok(()) => {
            // Post-commit: retire every variant of the OLD path — a slug
            // rename must kill the old URL's file and its alias copies
            // (index.md, {dir}.html, lang-prefixed), not just index.html.
            if let Some(p) = &pre {
                let eff_lang = if lang.is_empty() { p.lang.as_str() } else { lang };
                if f.slug != p.slug || eff_lang != p.lang {
                    for v in &p.files {
                        let _ = std::fs::remove_file(sc.out.join(v));
                    }
                }
            }
            let extra: Vec<(i64, String, String)> = pre
                .as_ref()
                .map(|p| p.terms.iter().map(|(t, s)| (f.id, t.clone(), s.clone())).collect())
                .unwrap_or_default();
            admin::regen_posts_with_terms(&sc, &[f.id], &extra);
            Redirect::to("/admin/posts").into_response()
        }
        Err(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

pub(crate) async fn post_data(State(app): State<App>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let cats = crud::term_slugs_for(&sc.db, id, "category").unwrap_or_default();
    let tag_names = crud::tag_names_for(&sc.db, id).unwrap_or_default();
    let lang = crud::lang_of(&sc.db, id).unwrap_or_default();
    /* Category checklist scope = the post's language: a term stays visible
       when (a) it is attached to this post, (b) a post of the same lang
       uses it, or (c) nothing uses it yet. */
    let allowed_cats: Vec<String> = (|| {
        let c = crate::db::conn(&sc.db).ok()?;
        let mut st = c.prepare(
            sql!("posts__select_allowed_category_slugs"),
        ).ok()?;
        let rows = st
            .query_map(rusqlite::params![lang, id], |r| r.get::<_, String>(0))
            .ok()?;
        rows.collect::<Result<Vec<String>, _>>().ok()
    })()
    .unwrap_or_default();
    let body = serde_json::json!({"cats": cats, "tags": tag_names, "lang": lang, "allowed_cats": allowed_cats}).to_string();
    ([(header::CONTENT_TYPE, "application/json")], body).into_response()
}

pub(crate) async fn clone_post(State(app): State<App>, headers: HeaderMap, Form(f): Form<CloneForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    match crud::clone_post(&sc.db, f.id, !f.draft.is_empty()) {
        Ok(new_id) => Redirect::to(&format!("/admin/edit/{new_id}")).into_response(),
        Err(e) => {
            eprintln!("clone_post {} failed: {e:#}", f.id);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// `/admin/new/{kind}` accepts either the section slug (`tours`) or the
/// kind itself (`tour`) — sidebar links use both spellings.
pub(crate) fn kind_from_section(s: &str) -> Option<&'static str> {
    plugins::kind_for_section(s).map(|k| k.kind).or_else(|| plugins::kind_def(s).map(|k| k.kind))
}

pub(crate) async fn new_post(State(app): State<App>, headers: HeaderMap, Path(kind): Path<String>, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Some(k) = kind_from_section(&kind) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // Default lang = site primary; the PLL "+" links carry ?lang=<code>
    // (validated against the configured set).
    let lang_cfg = crate::db::conn(&sc.db).map(|c| crate::lang::config(&c)).unwrap_or_default();
    let lang = q.get("lang")
        .filter(|l| lang_cfg.langs.contains(l))
        .cloned()
        .unwrap_or_else(|| lang_cfg.primary.clone());
    let f = crud::PostForm { kind: k.into(), lang, status: "draft".into(), body_json: "[]".into(), ..Default::default() };
    html_admin(&s, "New", &edit_html(sc, &s, &f), true, "post-php").into_response()
}

pub(crate) async fn edit_post(State(app): State<App>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    // Sync DB load + editor render off the reactor — gdlr builds are
    // 20-40ms of blocking work that would stall public traffic.
    let (s2, sc2) = (s.clone(), sc.clone());
    tokio::task::spawn_blocking(move || {
        match crud::load_post(&sc2.db, id) {
            Ok(Some(f)) => html_admin(&s2, "Edit", &edit_html(&sc2, &s2, &f), true, "post-php").into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
    })
    .await
    .unwrap_or_else(|e| {
        eprintln!("edit_post join: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}

/// WP classic TinyMCE editor (`#content` textarea + Visual/Code tabs).
/// `with_assets` controls the tinymce.min.js `<script>` tag — the gdlr
/// Page Builder branch already loads it, so it passes false. The caller's
/// output must place the returned markup where a tinymce script has run.
fn classic_editor(content: &str, with_assets: bool) -> String {
    let tinymce_tag = if with_assets {
        r#"<script src="/admin/wp/tinymce/tinymce.min.js"></script>"#
    } else {
        ""
    };
    format!(
        r###"<div id="postdivrich" class="postarea wp-editor-wrap">

<div id="wp-content-editor-tools">
<div id="media-buttons">
<a href="#" class="button insert-media add_media" data-editor="content" title="Add Media" onclick="mf_open('insert');return false"><span class="wp-media-buttons-icon"></span> Add Media</a>
</div>
<div class="wp-editor-tabs">
<button type="button" class="wp-switch-editor switch-tmce" id="content-tmce">Visual</button>
<button type="button" class="wp-switch-editor switch-html" id="content-html">Code</button>
</div></div>
<div id="wp-content-editor-container" class="wp-editor-container">
<div id="ed_toolbar" class="quicktags-toolbar"></div>
<textarea class="wp-editor-area" id="content" name="content" rows="18">{content}</textarea>
</div></div>
<link rel="stylesheet" href="/admin/wp/editor.min.css">
{tinymce_tag}<script>
// WP user-settings shims — tinymce 'wordpress'/'wp_adv' plugins read the
// editor's last mode / toolbar state through these globals.
window.getUserSetting = window.getUserSetting || function(n, d) {{ try {{ var v = localStorage.getItem('wp-' + n); return v === null ? d : v; }} catch (e) {{ return d; }} }};
window.setUserSetting = window.setUserSetting || function(n, v) {{ try {{ localStorage.setItem('wp-' + n, v); }} catch (e) {{}} }};
window.deleteUserSetting = window.deleteUserSetting || function(n) {{ try {{ localStorage.removeItem('wp-' + n); }} catch (e) {{}} }};
</script>
<script>

tinymce.init({{
  selector: '#content',
  skin_url: '/admin/wp/tinymce/skins/lightgray',
  theme: 'modern',
  theme_url: '/admin/wp/tinymce/themes/modern/theme.min.js',
  plugins: 'charmap colorpicker compat3x directionality fullscreen hr image link lists media paste tabfocus textcolor wpautoresize wpdialogs wpeditimage wpemoji wpgallery wptextpattern wpview wordpress',
  menubar: 'file edit view insert format tools table',
  branding: false,
  height: 460,
  toolbar1: 'formatselect bold italic strikethrough bullist numlist blockquote hr alignleft aligncenter alignright link unlink wp_more fullscreen wp_adv',
  toolbar2: 'forecolor pastetext removeformat charmap outdent indent undo redo wp_help',
  toolbar3: '',
  wp_autoresize_on: true,
  relative_urls: false,
  content_css: '/admin/wp/editor.min.css,/admin/wp/cms-overrides.css,/admin/wp/atrium.css'
}});
document.getElementById('content-html') && document.getElementById('content-html').addEventListener('click', function(){{ if (typeof tinymce !== 'undefined' && tinymce.get('content')) tinymce.get('content').hide(); }});
document.getElementById('content-tmce') && document.getElementById('content-tmce').addEventListener('click', function(){{ if (typeof tinymce !== 'undefined' && tinymce.get('content')) tinymce.get('content').show(); }});
// WP pins the media-buttons row + TinyMCE toolbar while scrolling the
// post editor (wp-pin-toolbar in editor.js). Sticky CSS can't do this —
// the toolbar's parent is only as tall as the toolbar — so pin via JS:
// .wp-pin gives the rows position:fixed and mirrors the wrap's width.
(function(){{
  var TOOLS_H = 36, BAR = 32;
  function refresh(){{
    var tools = document.getElementById('wp-content-editor-tools');
    if (!tools) return;
    var wrapEl = document.getElementById('postdivrich');
    if (!wrapEl) return;
    var grp = wrapEl.querySelector('.mce-toolbar-grp');
    var wr = wrapEl.getBoundingClientRect();
    var grpH = grp ? grp.offsetHeight : 0;
    // WP pins the rows only while the editor is in view; once the wrap's
    // bottom scrolls past the pinned area (e.g. into Page Builder) both
    // rows release back to their normal place.
    var pin = wr.top <= BAR && wr.bottom > BAR + tools.offsetHeight + grpH;
    tools.classList.toggle('wp-pin', pin);
    if (pin) {{
      var tr = tools.parentElement.getBoundingClientRect();
      tools.style.width = tr.width + 'px';
      tools.style.left = tr.left + 'px';
    }} else {{ tools.style.width = ''; tools.style.left = ''; }}
    // Toolbar pins just below the tools row — the row's height varies
    // (media buttons + tabs can wrap), so measure it instead of a fixed
    // offset.
    var toolsH = tools.offsetHeight;
    if (grp) {{
      grp.classList.toggle('wp-pin', pin);
      if (pin) {{
        grp.style.top = (BAR + toolsH) + 'px';
        grp.style.width = wr.width + 'px';
        grp.style.left = wr.left + 'px';
      }} else {{
        grp.style.top = ''; grp.style.width = ''; grp.style.left = '';
      }}
      wrapEl.style.paddingTop = pin ? grpH + 'px' : '';
    }}
    // The whole right sidebar pins under the adminbar while the editor
    // stays in view; it becomes its own scrollbar (taller than the
    // viewport — scroll it to reach the bottom boxes).
    var sb = document.getElementById('postbox-container-1');
    if (sb) {{
      var pb = document.getElementById('post-body');
      var pr = pb ? pb.getBoundingClientRect() : {{top: 0}};
      var pinS = pr.top <= BAR && wr.bottom > BAR;
      if (pinS && !sb.classList.contains('wp-pin')) {{
        window._sbH = sb.offsetHeight;
        window._sbL = sb.getBoundingClientRect().left;
      }}
      sb.classList.toggle('wp-pin', pinS);
      if (pinS) {{
        sb.style.left = window._sbL + 'px';
        sb.style.width = sb.parentElement.getBoundingClientRect().width + 'px';
        sb.style.height = 'calc(100vh - ' + BAR + 'px)';
      }} else {{
        sb.style.left = ''; sb.style.width = '';
        sb.style.height = ''; sb.style.minHeight = '';
      }}
    }}
  }}
  window.addEventListener('scroll', refresh, {{passive:true}});
  window.addEventListener('resize', refresh);
}})();
</script>
"###,
        content = esc(content),
        tinymce_tag = tinymce_tag
    )
}


pub(crate) fn edit_html(sc: &SiteCtx, s: &Sess, f: &crud::PostForm) -> String {
    let id = f.id.map(|n| n.to_string()).unwrap_or_default();
    let id_field = if id.is_empty() {
        String::new()
    } else {
        format!(r#"<input type="hidden" name="id" value="{id}">"#)
    };
    // Unregistered kind (plugin removed, row survived): label degrades to
    // the raw kind string — the record still renders, never post-ifies.
    let kind_label = spec_for_kind(&f.kind).map(|k| k.noun).unwrap_or(&f.kind);
    let add_kind = spec_for_kind(&f.kind).map(|k| k.kind).unwrap_or(&f.kind);
    let h1 = if f.id.is_some() {
        format!("Edit {kind_label}")
    } else {
        format!("Add New {kind_label}")
    };
    let menu = spec_for_kind(&f.kind).map(|k| k.section).unwrap_or("posts");
    /* WP permalink preview: an empty stored slug previews the slugified
       title (`get_sample_permalink`), never a bare `//`. */
    let preview_slug = if f.slug.is_empty() { crud::slugify(&f.title) } else { f.slug.clone() };
    let permalink = public_path(sc, &f.lang, &f.kind, &preview_slug);
    let permalink_base = {
        let origin = site_url(sc).trim_end_matches('/').to_string();
        let rel = permalink.strip_prefix(&origin).unwrap_or(&permalink);
        let p = rel.trim_end_matches('/');
        let base = if !preview_slug.is_empty() && p.ends_with(&preview_slug) {
            p[..p.len() - preview_slug.len()].to_string()
        } else {
            format!("{p}/")
        };
        format!("{}{}", origin, base)
    };
    let gdlr = crud::body_is_gdlr(&f.body_json);

    let assigned_cats: Vec<String> = f
        .terms
        .lines()
        .filter_map(|ln| {
            let mut p = ln.splitn(3, ':');
            let tax = p.next()?;
            let slug = p.next()?;
            if tax == "category" {
                Some(slug.to_string())
            } else {
                None
            }
        })
        .collect();
    let tags: String = f
        .terms
        .lines()
        .filter_map(|ln| {
            let mut p = ln.splitn(3, ':');
            let tax = p.next()?;
            let _slug = p.next()?;
            let name = p.next()?;
            if tax == "post_tag" {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let cats = crud::term_choices(&sc.db, "category", &assigned_cats).unwrap_or_default();
    let mut cat_html = String::new();
    for (slug, name, on) in cats {
        let chk = if on { " checked" } else { "" };

        cat_html.push_str(&format!(
            r#"<li><label><input type="checkbox" class="catcheck" value="{}:{}"{chk}> {}</label></li>"#,
            esc(&slug),
            esc(&name),
            esc(&name)
        ));
    }

    let editor = if gdlr {
        let conn = crate::db::conn(&sc.db);
        let gl = plugins::adapter_for_slug("goodlayers");
        let boot = conn.ok().as_ref().and_then(|c| {
            gl.and_then(|a| a.editor_boot(
                c, sc,
                f.id.unwrap_or(0),
                if f.body_json.trim().is_empty() { "[]" } else { &f.body_json },
                &s.csrf,
            ))
        }).unwrap_or_default();
        format!(
            r###"{classic}
<div class="postbox" id="gdlr-core-page-builder-meta"><h2>Page Builder</h2><div class="inside">
{boot}
</div></div>
{assets}
<script>
tinyMCEPreInit = {{ mceInit: {{}}, qtInit: {{}} }};
tinyMCEPreInit.mceInit['gdlr_core_tmce'] = {{
  selector: '#gdlr_core_tmce',
  skin_url: '/admin/wp/tinymce/skins/lightgray',
  theme: 'modern',
  theme_url: '/admin/wp/tinymce/themes/modern/theme.min.js',
  plugins: 'charmap colorpicker compat3x directionality fullscreen hr image link lists media paste tabfocus textcolor wpautoresize wpdialogs wpeditimage wpemoji wpgallery wptextpattern wpview wordpress',
  menubar: 'file edit view insert format tools table',
  branding: false,
  toolbar1: 'formatselect bold italic strikethrough bullist numlist blockquote hr alignleft aligncenter alignright link unlink wp_more fullscreen wp_adv',
  toolbar2: 'forecolor pastetext removeformat charmap outdent indent undo redo wp_help',
  toolbar3: '',
  relative_urls: false,
  body_class: 'post-type-page locale-en-gb'
}};
tinyMCEPreInit.qtInit['gdlr_core_tmce'] = {{ id: 'gdlr_core_tmce', buttons: 'strong,em,link,block,del,ins,img,ul,ol,li,code,more,close' }};
window.getUserSetting = window.getUserSetting || function(n, d) {{ try {{ var v = localStorage.getItem('wp-' + n); return v === null ? d : v; }} catch (e) {{ return d; }} }};
window.setUserSetting = window.setUserSetting || function(n, v) {{ try {{ localStorage.setItem('wp-' + n, v); }} catch (e) {{}} }};
window.deleteUserSetting = window.deleteUserSetting || function(n) {{ try {{ localStorage.removeItem('wp-' + n); }} catch (e) {{}} }};
</script>"###,
            boot = boot,
            assets = gl.and_then(|a| a.editor_assets(sc)).unwrap_or_default(),
            // Classic editor first (above Page Builder) — it loads
            // tinymce.min.js itself; the builder script tag below is a
            // harmless second include.
            classic = classic_editor(&f.content, true),
        )
    } else {
        classic_editor(&f.content, true)
    };
    let tag_cloud = {
        let mut c = String::new();
        if let Ok(list) = crud::term_choices(&sc.db, "post_tag", &[]) {
            for (i, (_s, name, _)) in list.iter().take(20).enumerate() {
                if i > 0 {
                    c.push_str(", ");
                }
                c.push_str(&format!(
                    r##"<a href="#" onclick="var h=document.getElementById('tagfield');var t={};var cur=h.value?h.value.split(',').map(function(s){{return s.trim()}}).filter(Boolean):[];if(cur.indexOf(t)<0)cur.push(t);h.value=cur.join(', ');renderTagChips();return false">{}</a>"##,
                    esc(&serde_json::to_string(&name).unwrap_or_else(|_| "\"\"".to_string())),
                    esc(name)
                ));
            }
            if list.len() > 20 {
                c.push_str(&format!(", <em>… ({} more)</em>", list.len() - 20));
            }
        }
        c
    };

    let tax_boxes = if f.kind == "post" {
        format!(
            r##"<div id="categorydiv" class="postbox"><h2>Categories</h2><div class="inside">
<ul class="category-tabs"><li class="tabs"><a href="#category-all">All Categories</a></li></ul>
<div id="category-all" class="tabs-panel" style="max-height:200px;overflow:auto">
<ul class="categorychecklist" id="catlist">{cat_html}</ul>
<input type="hidden" name="cat" id="catfield" value="">
</div>
<p><a href="#category-add" class="taxonomy-add-new" onclick="document.getElementById('cat-add-form').style.display='block';document.getElementById('newcat').focus();return false">+ Add Category</a></p>
<div id="cat-add-form" style="display:none">
<input type="text" id="newcat" placeholder="Category name">
<button type="button" class="button" onclick="
var n=document.getElementById('newcat');var name=n.value.trim();if(!name)return false;
var slug=name.toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-+|-+$/g,'')||name;
var li=document.createElement('li');
li.innerHTML='&lt;label&gt;&lt;input type=&quot;checkbox&quot; class=&quot;catcheck&quot; value=&quot;'+slug+':'+name.replace(/&quot;/g,'&amp;quot;')+'&quot; checked&gt; '+name.replace(/&lt;/g,'&amp;lt;')+'&lt;/label&gt;';
document.getElementById('catlist').appendChild(li);
n.value='';document.getElementById('cat-add-form').style.display='none';
return false">Add</button>
</div>
</div></div>
<div id="tagsdiv-post_tag" class="postbox"><h2>Tags</h2><div class="inside">
<div class="jaxtag">
<div class="ajaxtag hide-if-no-js">
<input type="text" id="new-tag" class="newtag" placeholder="tag, tag">
<button type="button" class="button tagadd" onclick="
var i=document.getElementById('new-tag'),h=document.getElementById('tagfield');
var cur=h.value? h.value.split(',').map(function(s){{return s.trim()}}).filter(Boolean) : [];
i.value.split(',').forEach(function(t){{t=t.trim();if(t&&cur.indexOf(t)<0)cur.push(t)}});
h.value=cur.join(', ');i.value='';renderTagChips();
return false">Add</button>
</div>
<div id="tagchips" class="tagchecklist"></div>
<input type="hidden" name="tags" id="tagfield" value="{tags}">
</div>
<p class="howto">Separate tags with commas</p>
<p><a href="#" onclick="document.getElementById('mostused-tags').style.display=document.getElementById('mostused-tags').style.display==='none'?'block':'none';return false">Choose from the most used tags</a></p>
<div id="mostused-tags" style="display:none">{tag_cloud}</div>
</div></div>"##,
            cat_html = cat_html,
            tags = esc(&tags),
            tag_cloud = tag_cloud
        )
    } else {
        String::new()
    };

    // WP Featured-image metabox: thumbnail + "Set/Remove featured image",
    // opening the media-frame picker (upload + library tabs).
    let feat_html = {
        let cur = f.featured_media_id.trim().parse::<i64>().unwrap_or(0);
        let thumb = if cur > 0 {
            crud::media_path(&sc.db, cur).ok().flatten().map(|p| {
                format!(r#"<img src="/wp-content/uploads/{}" style="max-width:100%;height:auto;display:block">"#, esc(&p))
            }).unwrap_or_default()
        } else {
            String::new()
        };
        let (has, none) = if thumb.is_empty() { ("display:none", "") } else { ("", "display:none") };
        format!(
            r##"<input type="hidden" name="featured_media_id" id="featured-id" value="{cur_val}">
<p id="set-featured-image" class="hide-if-no-js" style="{none}">
<a href="#" class="set-featured-image" onclick="mf_open();return false">Set featured image</a></p>
<p id="featured-thumb-wrap" style="{has}">
<a href="#" onclick="mf_open();return false" id="featured-thumb">{thumb}</a>
</p>
<p id="featured-hint" class="hide-if-no-js" style="color:#646970;font-size:12px;{has}">Click the image to edit or update</p>
<p id="featured-remove" style="{has}"><a href="#" class="remove-featured-image" style="color:#b32d2e" onclick="document.getElementById('featured-id').value='';document.getElementById('featured-thumb-wrap').style.display='none';document.getElementById('featured-hint').style.display='none';this.parentNode.style.display='none';document.getElementById('set-featured-image').style.display='';return false">Remove featured image</a></p>"##,
            cur_val = if cur > 0 { cur.to_string() } else { String::new() },
            none = none,
            has = has,
            thumb = thumb,
        )
    };
    let media_frame = super::admin::media_frame_html(&s.csrf);

    let status_label = match f.status.as_str() {
        "publish" => "Published",
        "future" => "Scheduled",
        "pending" => "Pending Review",
        "trash" => "Trash",
        _ => "Draft",
    };

    // WP publish box "Published on: <b>date</b> Edit" — the stored
    // "YYYY-MM-DD HH:MM:SS" fills a datetime-local (needs "T", and the
    // browser widget can't carry seconds).
    let pub_date = {
        // Stored `published_at` is UTC (see save_post_on's convention
        // note); the publish box speaks site-local wall time, so convert
        // through `settings.timezone` for display — the same value the
        // datetime-local input round-trips back on save.
        let tz = crate::db::misc::setting_value(&sc.db, "timezone", "UTC");
        let local = |d: &str| {
            crate::db::misc::utc_to_local(d, &tz).unwrap_or_else(|| d.to_string())
        };
        let d = if !f.published_at.is_empty() {
            f.published_at.clone()
        } else {
            f.id
                .and_then(|id| crud::published_at(&sc.db, id).ok().flatten())
                .unwrap_or_default()
        };
        if d.is_empty() { "immediately".to_string() } else { esc(&local(&d)) }
    };
    let pub_date_val = {
        let tz = crate::db::misc::setting_value(&sc.db, "timezone", "UTC");
        crate::db::misc::utc_to_local(&f.published_at, &tz)
            .unwrap_or_else(|| f.published_at.clone())
            .get(..16)
            .unwrap_or("")
            .replace(' ', "T")
    };

    // Revisions metabox — WP shows it only when the post actually has
    // snapshots; restore/compare screens live under /admin/revisions.
    let revisions_box = {
        let n = f.id.map(|id| crud::revision_count(&sc.db, id)).unwrap_or(0);
        if n > 0 {
            format!(
                r#"<div id="revisionsdiv" class="postbox"><h2>Revisions</h2><div class="inside"><a href="/admin/posts/{id}/revisions">Browse {n} revision{n_pl}</a></div></div>"#,
                n_pl = if n == 1 { "" } else { "s" },
            )
        } else {
            String::new()
        }
    };

    // Drafts/pending/scheduled preview through the session-gated render
    // route (6.1); published content keeps the public permalink. The
    // preview shows the *last saved* state — unsaved editor changes are
    // not reflected (documented scope cut).
    let (preview_href, preview_label) = match f.status.as_str() {
        "publish" => (permalink.clone(), "Preview Changes"),
        _ => (
            f.id.map(|id| format!("/admin/posts/{id}/preview"))
                .unwrap_or_else(|| permalink.clone()),
            "Preview",
        ),
    };
    let pub_label = if f.status == "future" { "Scheduled for:" } else { "Published on:" };

    // N-language metabox: one radio per configured language, one
    // translations row per language (self row = the post itself). The
    // picker field names are `trans_<lang>` — save reconciles the group.
    let lang_cfg = crate::db::conn(&sc.db).map(|c| crate::lang::config(&c)).unwrap_or_default();
    let lang_radios: String = lang_cfg
        .langs
        .iter()
        .map(|lg| {
            format!(
                r#"<label><input type="radio" name="lang" value="{}"{}> {} {}</label>"#,
                esc(lg),
                if f.lang == *lg { " checked" } else { "" },
                crate::lang::flag_emoji(lg),
                esc(&crate::lang::lang_label(lg)),
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut trans_rows = String::new();
    let mut trans_fields = String::new();
    for lg in &lang_cfg.langs {
        let other = f.translations.get(lg).map(|s| s.as_str()).unwrap_or("");
        let (action, flag_mark) = if f.lang == *lg {
            (
                f.id.map(|i| format!(r#"<a href="/admin/edit/{i}">Edit</a>"#)).unwrap_or_default(),
                "✓",
            )
        } else if other.is_empty() {
            (
                format!(r#"<a href="/admin/new/{add_kind}?from={id}&amp;lang={lg}">+ Add new</a>"#),
                "",
            )
        } else {
            (format!(r#"<a href="/admin/edit/{other}">Edit</a>"#), "")
        };
        trans_rows.push_str(&format!(
            "<tr><td>{}</td><td>{action}</td><td>{flag_mark}</td></tr>",
            crate::lang::flag_emoji(lg)
        ));
        if f.lang != *lg {
            trans_fields.push_str(&format!(
                r#"<input type="hidden" name="trans_{lg}" value="{}">"#,
                esc(other)
            ));
        }
    }

    let (tour, plugin_assets) = match plugins::editor_for(&f.kind) {
        Some(e) => ((e.metabox)(sc, s, f), (e.assets)()),
        None => (String::new(), ""),
    };
    format!(
        r###"{chrome}<div class="wrap">
<h1 class="wp-heading-inline">{h1}</h1>
<a href="/admin/{menu}" class="page-title-action">← {kind_label}s</a>
<a href="/admin/new/{add_kind}" class="page-title-action">Add {kind_label}</a>
<hr class="wp-header-end">
<form name="post" method="post" action="/admin/save" id="post">
<input type="hidden" name="csrf" value="{csrf}">
{id_field}
<input type="hidden" name="body_json" id="gdlr-json">
<input type="hidden" name="kind" value="{kind}">
<div id="poststuff">
<div id="post-body" class="metabox-holder columns-2">
<div id="post-body-content">
<div id="titlediv">
<div id="titlewrap"><input type="text" name="title" size="30" id="title" spellcheck="true" value="{title}" placeholder="Add title" required></div>
<div class="inside">
<div id="edit-slug-box" class="hide-if-no-js">
<strong>Permalink:</strong>
<span id="sample-permalink"><a href="{permalink}">{permalink}</a></span>
<span id="edit-slug-buttons">
<span id="permalink-edit" style="display:none"><code>{permalink_base}</code><input type="text" name="slug" id="post_name" value="{slug}" class="new-slug"></span>
<button type="button" class="button button-small" id="slug-edit-btn">Edit</button>
</span>
</div></div></div>
{editor}
{tour}
<div class="postbox"><h2>Excerpt</h2><div class="inside"><textarea name="excerpt" rows="3" style="width:100%">{excerpt}</textarea></div></div>

{plugin_assets}
</div>
<div id="postbox-container-1" class="postbox-container">
<div id="side-sortables" class="meta-box-sortables">
<div id="submitdiv" class="postbox"><h2>Publish</h2><div class="inside">
<div class="submitbox" id="submitpost">
<div id="minor-publishing">
<div id="misc-publishing-actions">
<div class="misc-pub-section misc-pub-post-status">Status: <span id="post-status-display">{status_label}</span>
<a href="#post_status" class="edit-post-status" onclick="document.getElementById('post-status-select').style.display='block';this.style.display='none';return false">Edit</a>
<div id="post-status-select" style="display:none">
<select name="status"><option value="draft" {draft}>Draft</option><option value="pending" {pending}>Pending Review</option><option value="future" {future}>Scheduled</option><option value="publish" {publish}>Published</option></select>
<a href="#post_status" class="save-post-status button" onclick="var s=this.parentNode;s.style.display='none';document.querySelector('.edit-post-status').style.display='';return false">OK</a>
</div></div>
<div class="misc-pub-section misc-pub-visibility">Visibility: <b>Public</b></div>
<div class="misc-pub-section curtime misc-pub-curtime">{pub_label} <b id="timestamp">{pub_date}</b>
<a href="#edit_timestamp" onclick="document.getElementById('timestampdiv').style.display='block';this.style.display='none';return false">Edit</a>
<div id="timestampdiv" style="display:none"><input type="datetime-local" name="published_at" value="{pub_date_val}">
<p class="howto">A future date schedules the post; it publishes automatically once the date passes (the next admin request flips it, its file regenerates, and the next full generate ships it).</p>
<a href="#edit_timestamp" class="button" onclick="document.getElementById('timestampdiv').style.display='none';this.previousElementSibling.previousElementSibling.style.display='none';document.querySelector('.misc-pub-curtime > a').style.display='';return false">OK</a></div></div>
</div></div>
<div id="major-publishing-actions">
<div id="delete-action"><button type="button" class="submitdelete deletion button-link" onclick="var b=new URLSearchParams();b.append('csrf','{csrf}');b.append('id','{id}');b.append('status','trash');fetch('/admin/trash',{{method:'POST',headers:{{'Content-Type':'application/x-www-form-urlencoded'}},body:b}}).then(function(){{location.href='/admin/posts'}});return false">Move to Trash</button></div>
<div id="publishing-action"><a href="{preview_href}" target="_blank" class="button button-secondary button-large" id="preview-btn" data-draft="{is_draft}" style="margin-right:8px">{preview_label}</a><button class="button button-primary button-large" id="publish" type="submit">Update</button></div>
</div>
</div>
</div></div>
<div id="ml_box" class="postbox"><h2>Language</h2><div class="inside">
<p class="pll-langs">{lang_radios}</p>
<p><b>Translations</b></p>
<table class="widefat">{trans_rows}</table>
{trans_fields}
</div></div>
{rankmath}
{revisions_box}
{tax_boxes}
<div id="postimagediv" class="postbox"><h2>Featured image</h2><div class="inside">
{feat_html}
</div></div>
</div></div></div></div>
</form></div>
<script>
function renderTagChips() {{
  var h = document.getElementById('tagfield');
  var box = document.getElementById('tagchips');
  if (!h || !box) return;
  box.innerHTML = '';
  var tags = h.value ? h.value.split(',').map(function(s){{return s.trim()}}).filter(Boolean) : [];
  tags.forEach(function(t) {{
    var span = document.createElement('span');
    span.className = 'ntdelbutton';
    span.style.cssText = 'display:inline-block;margin:2px 4px 2px 0;padding:2px 22px 2px 8px;border:1px solid #c3c4c7;border-radius:2px;background:#f6f7f7;position:relative';
    span.textContent = t;
    var x = document.createElement('a');
    x.href = '#'; x.textContent = '×';
    x.style.cssText = 'position:absolute;right:4px;top:1px;text-decoration:none;font-weight:bold';
    x.onclick = function(ev) {{
      ev.preventDefault();
      var cur = h.value.split(',').map(function(s){{return s.trim()}}).filter(Boolean);
      h.value = cur.filter(function(s){{return s!==t}}).join(', ');
      renderTagChips();
    }};
    span.appendChild(x);
    box.appendChild(span);
  }});
}}
renderTagChips();
(function() {{
  var btn = document.getElementById('slug-edit-btn');
  if (!btn) return;
  btn.addEventListener('click', function() {{
    var s = document.getElementById('sample-permalink'),
        e = document.getElementById('permalink-edit'),
        n = document.getElementById('post_name');
    s.style.display = 'none';
    btn.style.display = 'none';
    e.style.display = 'inline';
    if (document.getElementById('slug-ok')) return;
    var ok = document.createElement('button');
    ok.id = 'slug-ok'; ok.type = 'button'; ok.className = 'button button-small save-slug';
    ok.textContent = 'OK';
    var cancel = document.createElement('a');
    cancel.href = '#'; cancel.className = 'button-cancel'; cancel.textContent = 'Cancel';
    e.parentNode.appendChild(ok);
    e.parentNode.appendChild(cancel);
    function close() {{
      e.style.display = 'none';
      s.style.display = '';
      btn.style.display = '';
      ok.remove(); cancel.remove();
    }}
    ok.onclick = function() {{
      var base = document.querySelector('#permalink-edit code').textContent;
      s.querySelector('a').textContent = base + n.value + '/';
      close();
    }};
    cancel.onclick = function(ev) {{ ev.preventDefault(); close(); }};
  }});
}})();
(function() {{
  /* Draft preview = WP semantics: save the form silently, THEN open the
     rendered preview — the link alone only shows the last saved state. */
  var btn = document.getElementById('preview-btn');
  var form = document.getElementById('post');
  if (!btn || !form || btn.dataset.draft !== '1') return;
  btn.addEventListener('click', function(ev) {{
    ev.preventDefault();
    btn.textContent = 'Saving preview…';
    /* run the submit listeners (TinyMCE save, gdlr canvas → #gdlr-json,
       cat checklist → hidden field) without navigating away */
    form.dispatchEvent(new Event('submit', {{cancelable: true}}));
    var body = new URLSearchParams(new FormData(form));
    fetch(form.action, {{method: 'POST', credentials: 'same-origin',
      headers: {{'Content-Type': 'application/x-www-form-urlencoded'}}, body: body}})
      .then(function(r) {{
        btn.textContent = 'Preview';
        if (r.ok || r.redirected) {{ window.open(btn.href, '_blank'); }}
        else {{ alert('Save failed — preview shows the last saved version.'); window.open(btn.href, '_blank'); }}
      }})
      .catch(function() {{
        btn.textContent = 'Preview';
        window.open(btn.href, '_blank');
      }});
  }});
}})();
(function() {{
  var form = document.getElementById('post');
  var field = document.getElementById('catfield');
  if (!form) return;
  form.addEventListener('submit', function() {{
    if (typeof tinymce !== 'undefined' && tinymce.get && tinymce.get('content')) {{ tinymce.get('content').save(); }}
    if (!field) return;
    var vals = [];
    var boxes = form.querySelectorAll('input.catcheck:checked');
    for (var i = 0; i < boxes.length; i++) vals.push(boxes[i].value);
    field.value = vals.join('\u001f');
  }});
}})();
</script>

{media_frame}
"###,
        media_frame = media_frame,
        chrome = chrome(menu, s),
        csrf = s.csrf,
        kind = esc(&f.kind),
        title = esc(&f.title),
        permalink = esc(&permalink),
        permalink_base = esc(&permalink_base),

        slug = esc(&preview_slug),
        excerpt = esc(&f.excerpt),
        rankmath = rankmath::metabox_html(sc, s, f),
        draft = if f.status == "draft" { "selected" } else { "" },
        pending = if f.status == "pending" { "selected" } else { "" },
        future = if f.status == "future" { "selected" } else { "" },
        publish = if f.status == "publish" { "selected" } else { "" },
        status_label = status_label,
        feat_html = feat_html,
        pub_date = pub_date,
        pub_date_val = esc(&pub_date_val),
        pub_label = pub_label,
        preview_href = esc(&preview_href),
        preview_label = preview_label,
        is_draft = if f.status == "publish" { "0" } else { "1" },
        revisions_box = revisions_box,
        lang_radios = lang_radios,
        trans_rows = trans_rows,
        trans_fields = trans_fields,
    )
}

pub(crate) async fn save_post(State(app): State<App>, headers: HeaderMap, Form(f): Form<crud::PostForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    // A malformed metabox payload must fail the whole save — otherwise the
    // post commits while its metas stay stale with no signal.
    let rm_raw = f.rank_math.trim();
    if !rm_raw.is_empty() && serde_json::from_str::<serde_json::Value>(rm_raw).is_err() {
        return (StatusCode::BAD_REQUEST, "invalid rank_math payload").into_response();
    }
    // Capture the old output path + term set BEFORE the save so a slug /
    // lang / kind change can delete the stale file at the old URL and
    // invalidate the archives the post is leaving.
    let pre = f.id.and_then(|id| snapshot_before_save(&sc, id));
    match crud::save_post(&sc.db, &f) {
        Ok(id) => {
            // nav-menus.php auto-add: a brand-new top-level page joins every
            // menu flagged "Automatically add new top-level pages".
            if f.id.is_none() && f.kind == "page" {
                let _ = crud::menus::auto_add_page(&sc.db, id);
            }
            // Write-back: the post file must hold the just-saved version;
            // a slug/lang/kind change kills the old URL's whole variant
            // set (index.md, {dir}.html, per-lang copies), not just its
            // index.html.
            if let Some(p) = &pre {
                if f.slug != p.slug || f.lang != p.lang || f.kind != p.kind {
                    for v in &p.files {
                        let _ = std::fs::remove_file(sc.out.join(v));
                    }
                }
            }
            let extra: Vec<(i64, String, String)> = pre
                .as_ref()
                .map(|p| p.terms.iter().map(|(t, s)| (id, t.clone(), s.clone())).collect())
                .unwrap_or_default();
            admin::regen_posts_with_terms(&sc, &[id], &extra);
            Redirect::to(&format!("/admin/edit/{id}")).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, format!("{e}")).into_response(),
    }
}


/// `GET /admin/posts/{id}/preview` — WP's "Preview" for non-published
/// posts (6.1). Renders through the same pipeline the generator uses but
/// accepts any status (draft/pending/future) and forces
/// `X-Robots-Tag` + `<meta name=robots>` to noindex so a shared preview
/// URL can never be crawled. Shows the *last saved* state — unsaved
/// editor changes are not previewed (documented scope cut).
pub(crate) async fn preview_post(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    match crate::render::render_preview(&sc.db, &sc.templates, id, &site_url(sc), crate::admin::tour_base(sc)) {
        Ok(html) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::HeaderName::from_static("x-robots-tag"), "noindex"),
            ],
            html,
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            format!("preview unavailable: {e}"),
        )
            .into_response(),
    }
}

