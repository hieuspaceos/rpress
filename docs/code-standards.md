# Code standards — rpress

## Hard rules

1. **SQL ownership is `src/db/` first.** New SQL must live in `src/db/sql/*.sql`
   and be loaded via `sql!("name")`, or in a narrow `src/db/*` helper when the
   query is generated or purely local to a data helper. New admin handlers must
   call `db::` functions instead of embedding SQL. Existing ported plugin/admin
   code is being migrated behind helpers as it is touched.

   Allowed exceptions must stay narrow and documented in code review: plugin
   adapter modules may call `sql!(...)` directly when preserving upstream
   WordPress plugin semantics, and admin handlers may own transaction scope
   while individual mutations still go through `db::` helpers.
2. **All POSTs check CSRF** via `check_csrf(&s, &f.csrf)` (`403` on mismatch).
   Handlers follow the shape: `need_auth` → `check_csrf` → mutate →
   `Redirect::to`.
3. **HTML is `format!` + `r#""#`/`r##"..."##` raw strings**, escaped via
   `esc()` for every interpolated user/DB value. A literal `"#` inside the
   template (e.g. `href="#..."`) kills `r#` — use `r##`.
4. **Forms never nest.** Controls in table rows bind via `form="form-id"`;
   the `<form>` element sits outside the table. This is why Quick Edit works.
5. **Repeated keys need unique names.** `Form<HashMap<String,String>>`
   collapses `ids=1&ids=2`. Use the `prefix-value` convention: `p-{id}`
   (posts), `d-{id}` (term deletes), `bcat-{slug}` (bulk cats), `ord-{id}`,
   `parent-{id}` (menus). Server strips with `strip_prefix`.
6. **Term mutations go through `attach_term`/`term_name`/`slugify`**
   (`db/mod.rs`). Never re-implement the slug→name lookup or slugification.
7. **Bulk edit is additive** — cats/tags insert only; status updates only on
   an explicit non-empty value. Quick edit replaces `category`/`post_tag`
   terms but preserves other taxonomies.
8. **Warnings = 0.** `cargo build` must emit none. Unused vars get `_`
   prefix; dead code gets deleted, not `#[allow]`ed.
9. **No site/domain literals in `src/`.** Site identity comes from CLI args
   (`SiteCtx`) or `settings.*` in that site's DB (`site_url`, `site_name`,
   `legacy_origin`). Seeded content that is the reference theme's lives in
   `src/seed/*.json` (loaded via `include_str!`), never inline `r#"..."#`
   blobs in `src/*.rs`.
10. **WP→rpress bootstrap is `rpress migrate <dump>`.** One command runs
    import → admin creation (random password, printed once) → generate.
    Keep it thin: it calls `import::run`, `auth::add_user`, `generate::run` —
    no new logic inside the command.

## Naming

- Handlers: WP-verb style — `tax_save`, `quick_edit`, `bulk_posts`,
  `clone_post`, `menus_save`. GET screens end in `_page`.
- Routes mirror WP: `/admin/posts`, `/admin/edit/{id}`, `/admin/tax/{tax}`,
  `/admin/tax/save`, `/admin/quickedit`, `/admin/bulk`.
- DB fns are verbs: `load_post`, `save_term`, `list_terms`, `attach_term`.
- Form structs mirror the handler: `TrashForm`, `CloneForm`,
  `QuickEditForm`, `TaxForm`, `TaxDelete`.

## Structure per handler

```rust
async fn x(State(app): State<App>, headers: HeaderMap, Form(f): Form<F>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    // mutate via db::*(&sc.db, ...), then Redirect::to(...)
}
```

## Screens

- Every admin screen composes `chrome(active_section, &s)` + page HTML and
  wraps in `html_admin(&s, title, body, htmx, screen_class)`.
- List screens reuse `render_list`; taxonomy screens reuse `tax_page_html`.
- Inline edit rows: hidden `<tr class="inline-edit-row">` template moved by
  JS under the target row; Cancel restores `display`, never removes the row.
- Content-type screens (post/page/tour) all dispatch through
  `KINDS`/`KindSpec` — section, kind, noun, list title, thead, and the extra
  `<td>` columns live in one registry entry per CPT (`admin/posts.rs`).

## Status vocabulary

`publish | future | draft | pending | trash`. Validate before writing —
`bulk_apply`/`quick_update`/`save_post` reject anything else. `future` is
scheduled: `published_at` in the future; `publish_due` (called from
`get_sess`) flips due rows to `publish` and regenerates their files.

## Do NOT

- Don't add REST JSON endpoints per screen — admin is server-rendered. The
  only public JSON is `/api/v1/*`.
- Don't read `terms` for display names in admin lists when `post_terms.name`
  already carries it (keeps list queries single-level).
- Don't add a second session store. Login sessions persist in the `sessions`
  table; `app.sessions` is only the hot in-memory cache and CSRF owner.
- Don't add a second path to a screen: post/page/tour differ only inside
  `KINDS` (`admin/posts.rs`). A new CPT gets a `KindSpec` row — not a new
  handler, list screen, or editor.
- Don't write `app.db`/`app.templates` — state is per-site on `SiteCtx`
  (`sc.db`, `sc.templates`, `sc.out`). Handlers get `sc` from `need_auth`.
- Don't let `www/nav.js` hold per-page state at module scope without an
  idempotence guard — it re-executes on every SPA swap. Same rule for
  theme init that binds to DOM nodes (`superfish`, `sf_menu_mod`): call
  it again after `navigate()` replaces the content root.
- Don't emit `[read more]`/`[gdlr_*]`/`[caption]` raw into rendered HTML —
  they go through `expand_body_shortcodes`/`rmwr_wrap` in
  `render_post::filter_content`; a literal bracket in output is a bug.
