# Baseline UI Audit — rpress admin (atrium skin)

Audit of `http://localhost:8410/admin` at 1440×900, authenticated session, all 12 screens screenshotted.
Purpose: input for a CSS-only design-token polish pass. No markup/behavior changes proposed.

Stylesheets audited: `/admin/wp/atrium.css` (1597 lines), `/admin/wp/cms-overrides.css` (~150 lines).
Load order confirmed (`src/admin/mod.rs:1159-1177`): vendored WP css → `colors.min.css` → `cms-overrides.css` → `atrium.css`.

Severity legend: **VA** = visual annoyance (user-visible defect), **P** = polish (visible inconsistency), **M** = minor (code-health / edge case).

---

## A. Global / design-token issues

| # | Sev | Finding | Evidence | Fix direction |
|---|-----|---------|----------|----------------|
| A1 | **VA** | `a.page-title-action` ("Add Post", "Add New", "Add New User") renders in **stock WP blue** (`rgb(56,88,233)` border + text, transparent bg, `border-radius:2px`) — completely off-palette vs the ink/paper/atrium button system. | `colors.min.css` `.wrap .page-title-action { border:1px solid var(--wp-admin-theme-color); border-radius:2px }` has higher specificity than atrium's `a.page-title-action` (0,1,1 vs 0,1,1 grouped under `.wp-core-ui .button` 0,2,0 for other buttons — the pta selector is only 0,1,1). Measured: bg `rgba(0,0,0,0)`, border `1px solid rgb(56,88,233)`, br `2px`. | Add `.wrap a.page-title-action` (or `.wp-core-ui a.page-title-action`) to the atrium button block so it gets `--r-sm`, `--paper` bg, `--ink-2` text. |
| A2 | **P** | No `:focus-visible` style for plain links, `.nav-tab`, `.subsubsub a`, `.row-actions a`, `.wp-submenu a`, `.ab-item`, checkbox/radio already covered but only via `:focus` (fires on mouse click too — visible orange outline on click is an annoyance). Verified: zero `a:focus-visible` rules across all loaded sheets. | atrium.css has `:focus-visible` only on `.wp-core-ui .button`; `:focus` used for inputs (`outline: 2px solid color-mix(in oklab, var(--focus) 35%, transparent)`) and checkboxes (`:focus { outline: 2px solid var(--focus) }`). | Add a global `.wp-admin a:focus-visible, .wp-admin .nav-tab:focus-visible, .wp-admin .subsubsub a:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; border-radius: var(--r-sm) }`; switch checkbox `:focus` → `:focus-visible`. |
| A3 | **P** | Hover on table row titles applies `font-style: italic`, which changes glyph metrics → row text reflows/shifts on hover. | atrium.css: `.wp-list-table td.column-title a:hover, .wp-list-table td.column-name a:hover { color: var(--accent); font-style: italic; }` | Drop italic; color change alone (or underline). |
| A4 | **M** | Literal `|` separators still render inside the pill-styled `subsubsub` filter list: "All (1140) **\|** … Published (951) **\|** …" — dead chrome inside a pill bar that already spaces items via `gap:4px`. | Screenshot /admin/posts: li innerText `All (1140) |`; atrium.css `ul.subsubsub li a { padding:3px 11px; border-radius: var(--r-pill) }`. | Hide the separator text node (`.subsubsub li::after` pattern is vendored WP `|`) or make the `|` `color:transparent`/`display:none`-equivalent without markup change, e.g. `ul.subsubsub li::after{display:none}` if vendored, or shrink li font and rely on pill bg. |
| A5 | **P** | `.button-primary` has no `transition` — `filter: brightness(1.05)` on hover snaps instantly, while every secondary button animates `background-color`/`border-color`/`translate`. | atrium.css `.wp-core-ui .button-primary` block: sets gradient + shadow, no `transition`; hover uses `filter: brightness(1.05)`. | Add `transition: filter var(--dur-micro) var(--ease-out), translate var(--dur-micro) var(--ease-out)`. |
| A6 | **M** | Redundant/conflicting declarations: `#wpadminbar` and flyouts repeat `background: var(--glass-strong); background: var(--glass-solid); background: var(--glass-strong);` (solid sandwiched mid-rule as "fallback" — works but the same override is re-declared ~40 lines later for `.ab-sub-wrapper`). `.notice`/`.notice-success`/`.updated`/`.message` each re-declare `border-left: 0`. | atrium.css top-bar block; notice block: `.notice { border-left: 0 }`, `.notice-success, .updated { border-left: 0 }`, `body.login .message { border-left:0 }`, `#login_error{border-left:0}`. | Consolidate fallback pattern (define once per component); drop repeat `border-left:0`s. |
| A7 | **P** | `cms-overrides.css` leaks hard-coded off-palette colors: `#wp-content-editor-tools.wp-pin { background:#f0f0f0; border-bottom:1px solid #e5e5e5 }`, `.mce-toolbar-grp.wp-pin { background:#fff }`, `#att-modal > div { box-shadow:0 5px 30px rgba(0,0,0,.4) }`, `.wp-caption { background:#f1f1f1 }`, `.wp-caption dd { color:#444 }`. On atrium paper these read as stark white/gray strips when the editor toolbar pins. | cms-overrides.css editor + modal blocks; atrium.css already overrides `#wp-content-editor-tools.wp-pin`/`mce-toolbar-grp.wp-pin` background but `#fff` for `.mce-toolbar-grp.wp-pin` in cms-overrides vs `var(--paper)` — atrium wins on order only, fragile. | Replace literals with `var(--paper-2)`/`var(--rule)`/`var(--shadow-float)` tokens. |
| A8 | **M** | The "MOTION pass" block writes a naked `transition-duration: var(--dur-short), var(--dur-micro), var(--dur-micro)` list onto `.wp-core-ui .button`, `.menu-top a`, `.subsubsub a`, `.nav-tab` — duration count is positional and silently misaligns with each element's actual `transition-property` list; for `.button` (4 properties) the 4th duration wraps. | atrium.css "MOTION pass" near EOF. | Either drop the block (durations already set per-component) or set `transition: all var(--dur-short) var(--ease-out)` deliberately. |
| A9 | **P** | Focus ring on text inputs uses `outline:2px solid color-mix(...35%)` (soft wash) while buttons use `outline:2px solid var(--focus)` (full) — two ring intensities; checkbox uses a third (`offset:1px`). | atrium.css form fields vs buttons vs checkbox blocks. | One `--focus-ring` token (style+offset) applied everywhere. |
| A10 | **M** | `body.wp-admin::before` grain overlay sits at `z-index:99998` — above the dock flyouts' `z-index:99999` is fine, but it also floats over media modal dialogs (`#mf-frame` z-index unknown → grain may raster over modal). Cheap opacity .045 but it is a full-viewport `feTurbulence` SVG repainted fixed. | atrium.css body grain block. | Verify against modal z-index; consider `z-index` below modals or `pointer-events:none` (already set) + `contain: paint`. |

## B. Component issues

### Buttons
| # | Sev | Finding | Evidence | Fix |
|---|-----|---------|----------|-----|
| B1 | VA | "Add New"-style buttons are WP-blue outlined (see A1) — inconsistent with every other button which is paper-fill, `--rule-2` border, `--r-sm` radius. | measured: `br:2px, border:1px solid rgb(56,88,233), bg:transparent` vs secondary `br:9px` | A1 fix. |
| B2 | P | Hidden quick-edit buttons (`#btn-ok`, Cancel) ship `pad:0px 16px` — reveal OK; but **visible** primary vs secondary heights differ across screens: "Save Menu" (menus) = 40px, "Apply" (posts) = 32px, "Log In" (login) = full-width 40px. Same class `.button` produces two sizes depending on vendored `.button-large` markup — CSS-only fix can't unify markup but can normalize `.button` min-height. | measured heights 32 vs 40. | `.wp-core-ui .button:not(.button-large)` uniform `min-height:32px`; document large variant as intentional. |
| B3 | M | Disabled pager buttons (`« ‹ › »`) sit at `color:var(--muted); border-color:var(--rule)` — readable but the whole `tablenav-pages` group still shows a colored `«`/`›` text accent on some screens (Users screenshot shows faint arrows). | atrium `.button.disabled` rule. | Add `pointer-events:none` visual cue (cursor) + lower opacity to `.tablenav-pages .button:disabled`. |

### Tables
| # | Sev | Finding | Evidence | Fix |
|---|-----|---------|----------|-----|
| B4 | VA | **Empty-state gap**: `/admin/contact` renders a floating glass table card with header row (Title/Shortcode/Submissions/Date/Lang) and *zero body rows* — a wide blank card, no "No forms found" row. | screenshot /admin/contact; `forms` table count = 0. | Ensure `tr.no-items` row is emitted (markup) — CSS can only style it; if emitted, atrium's `.wp-list-table tr.no-items td` is fine. If rows truly absent → template bug, out of CSS scope; flag it. |
| B5 | P | Description column is unbounded — on `/admin/tax/region` one term's multi-paragraph description makes a single row ~2600px tall; the card scrolls forever and dock covers the tail. On `/admin/tax/category` same cell wraps 5 lines mid-row. | screenshot /admin/tax/region. | `.wp-list-table td.column-description { max-width:…; }` + `-webkit-line-clamp:6` with ellipsis, or `max-height` overflow. |
| B6 | P | Row heights balloon to 127–147px on /admin/posts (measured) driven by the third-party "SEO Details" cell (badge + 3 stacked lines). Whole table reads as airy cards, not a list. | measured rowHeights [127,147,127]. | Cell content isn't tokenable, but `td { vertical-align: top }` consistency + `line-height` tightening reduces perceived bloat; optionally `max-width` on SEO col. |
| B7 | M | Zebra striping is explicitly killed: `.wp-list-table .striped > tbody > :nth-child(odd){background:transparent}` — so `.striped` tables lose their affordance; only `:hover` row bg (`--paper-2`) differentiates rows. Combined with `td{border-top:1px solid var(--rule)}` light rule this is acceptable on paper but verify contrast vs glass bg at `oklch(90% …)` — rule on glass ~68% opaque over aurora gradient can nearly vanish. | atrium list-table block. | Keep kill (intentional) but bump `--rule` slightly darker or add `tbody tr + tr` border via `--rule-2`? Prefer: `border-top-color: var(--rule)` → `oklch(87%…)` test. |
| B8 | P | Header row uses `--font-num` 10px letterspaced labels — elegant, but `Count`/num columns are right-aligned text while header letters are `.1em` tracked mono; alignment of "Count" header vs right-aligned column is fine, yet the *checkbox* `th`/`td` cell in `check-column` is `width:100%` (measured 1368px on comments when other cols hidden). | comments DOM dump: `check-column` w=1368 when all data cols `.hidden`. | `.wp-list-table .check-column { width:2.2em }` hard pin — vendored sets this but class `.hidden` columns collapse table to cb-only (see C4). |
| B9 | M | Row-actions fade `opacity 0→1` on hover only; `tr:focus-within` covered (good). But `.row-actions` links are `--muted` 500-weight → on hover row-bg `--paper-2` contrast is marginal. | atrium `.row-actions a { color:var(--muted) }` | `--ink-2` for row-action links, keep muted for separators. |

### Forms
| # | Sev | Finding | Evidence | Fix |
|---|-----|---------|----------|-----|
| B10 | P | Disabled inputs are invisible as disabled: Settings → General shows Site Title/Tagline/URL as plain text lines (no control chrome at all — actually `readonly` styled borderless?) while Timezone/WPLANG selects are pale and Time Format radios render as tiny unchecked circles with no label emphasis. Gray-on-paper disabled select has `--muted` text but identical bg to enabled. | screenshot /admin/settings/general. | `input:disabled, select:disabled { background:var(--paper-2); color:var(--muted); border-color:var(--rule) }`. |
| B11 | P | Checkbox affordance too subtle: 15px, `1.5px solid --rule-2` on paper = nearly invisible unchecked state on login "Remember Me" and bulk-select column. Also `rotate:90deg` checked animation rotates a symmetric square — invisible effect, wasted motion. | measured 16×15 login checkbox; atrium checkbox block `rotate:90deg` on `:checked`. | Darken unchecked border (`--muted`), drop the rotate (draw a check via ::before mask instead of `content:none`). |
| B12 | M | `select`/`input` height inconsistent: tax page "Parent Región" select is ~32px but narrower than sibling text inputs (270px vs 340px) — misaligned column of controls. "Add Category" primary button 40px while inputs 32px — deliberate hierarchy, fine. | screenshots /admin/tax/*. | Uniform `min-height:32px` on `.wp-admin select`; let widths vary but give selects `min-width` matching input baseline. |
| B13 | M | Search inputs on desktop are `--r-sm` (9px) squared; on mobile they morph to `--r-pill` + glass bg + shadow — deliberate responsive redesign, but pill-vs-square also diverges *within* desktop: tablenav search = square, no glass. Consider tokenizing `search` variant. | atrium `.tablenav input[type=search]{height:30px;padding:4px 9px}` vs mobile block `border-radius:var(--r-pill)`. | Optional unify: keep desktop square, fine; flag for token `--control-radius`. |

### Notices
| # | Sev | Finding | Evidence | Fix |
|---|-----|---------|----------|-----|
| B14 | P | `.notice` padding `10px 14px` but glass card is `--r-md` radius — text sits 14px from a 13px-radius corner, fine; the decorative `::before` chip (8px rotated square) overlaps text baseline alignment (`vertical-align:1px`) — on dashboard warning notice the chip is ~1px high vs text baseline, looks slightly high. | screenshot dashboard; atrium `.notice::before{vertical-align:1px}`. | `vertical-align:0` or `baseline` + `translate:0 -1px`. |
| B15 | M | `atrium-drop-in` animation replays on every page (single iteration, ok) but `div.error`/`updated` also animate — combined with `position` shifts could reflow; acceptable. | atrium keyframes. | Keep; ensure `prefers-reduced-motion` kill covers it (it does via global rule). |

### Sidebar / dock / adminbar
| # | Sev | Finding | Evidence | Fix |
|---|-----|---------|----------|-----|
| B16 | P | Dock overlaps content at intermediate scroll positions (dashboard: dock top=816px, content flows beneath until final 120px pad). Text visible *through* glass behind dock creates legibility noise — glass `68%` opacity + blur(14px) is readable but the last line "Découvrez nos budgets…" renders partially obscured. | measured dockTop 816 vs vh 900; screenshot dashboard bottom-right. | Increase `--glass` opacity (68%→~80%) or add `background:var(--glass-strong)` to `#adminmenu` already done; alternative: `padding-bottom` already 120px — accept overlap mid-scroll as designed, bump glass to `82%`. |
| B17 | M | Adminbar measured 33px not 32px (border-bottom adds 1px) — `.wp-pin` offsets in cms-overrides assume `top:32px` (editor tools, sidebar pin) → 1px seam where pinned toolbar's `top:32px` meets a 33px bar → sub-pixel gap/overlap on pin. | measured `#wpadminbar` h=33; cms-overrides `top:32px`. | `#wpadminbar{box-sizing:border-box;height:32px}` or pin offsets `top:33px`. |
| B18 | P | Menu flyouts appear/disappear via `display:none→block` — zero transition; every other element animates ≤160ms. | atrium `.wp-submenu{display:none}` + `:hover{display:block}`; only `.ab-submenu` (top bar) gets opacity transition. | Use `visibility`/`opacity`/`translate` toggle instead of display (pure CSS, no markup change). |
| B19 | M | `#collapse-button` "+" tile uses dashed `--rule-2` border 1.5px — unique border style nowhere else; fine as accent, but its `:hover` lift `translateY(-8px) scale(1.08)` has no `:focus-visible` style (button element). | atrium collapse-button block. | Add `:focus-visible{outline:2px solid var(--focus)}`. |
| B20 | M | `#wpfooter` padding `10px 20px 10px 0` → footer text starts at viewport x=0 while `.wrap` content starts at 24px — misaligned left edge. | measured footerLeftX=0 vs wrapPadLeft 24px. | `#wpfooter{padding-left:24px}` or match `.wrap` gutters. |

### Pagination / tabs
| # | Sev | Finding | Evidence | Fix |
|---|-----|---------|----------|-----|
| B21 | M | Pager `tablenav-pages` mono 10-11px vs body 13px — deliberately small, but page-number input `#current-page-selector` at 11px is tiny vs 32px control height (measured). | atrium `.tablenav-pages{font-size:11px}`. | 12px mono; input `height:30px` consistent with selects. |
| B22 | P | `.nav-tab-active` uses `inset 0 2px 0 var(--accent)` top accent + `--paper` bg — page bg is `--paper` *under* aurora gradient + glass, so active tab's `--paper` solid bg looks *lighter* than surrounding area → active tab reads as raised white chip rather than "connected" tab. | atrium nav-tab block; screenshot edit screen tabs. | Use `var(--glass)` on active tab to match table surfaces, keep accent bar. |

## C. Per-screen issues

### 1. /admin — Dashboard
- **C1.1 P** — "At a Glance" heading renders 14px `Space Grotesk` (measured) vs atrium's intended 13px postbox headers — vendored `.postbox h2` (14px) beats atrium `.postbox>h2`? All widget headers 14px, consistent but off-spec. Fix: bump atrium header rule specificity.
- **C1.2 P** — Dock glass overlaps Activity widget tail text mid-scroll (B16).
- **C1.3 M** — Welcome/warning notice chip `::before` sits high (B14).
- **C1.4 M** — `.meta-box-sortables` padding 0 → postboxes butt directly to viewport edge-adjacent margins; consistent, but the first postbox top edge touches the `h1` baseline zone tightly (screenshot: ~8px gap between "Dashboard." h1 and first card vs 20px between cards). Fix: `margin-top` on first `.postbox` or `.meta-box-sortables > .postbox:first-child{margin-top:0}` normalize.

### 2. /admin/posts
- **C2.1 VA** — "Add Post" button WP-blue (A1/B1).
- **C2.2 P** — Both "All" *and* "Mine" pills carry `.current` → two black pills, ambiguous state (markup emits both; CSS could at minimum `.current ~ .current` de-emphasize, but real fix is template logic — flag). Evidence: DOM dump `aCls:"current"` on first two pills.
- **C2.3 P** — Pipe separators inside pills (A4): "All (1140) **|**".
- **C2.4 M** — Row heights 127–147px (B6); "SEO Details" column stacked lines.
- **C2.5 M** — Row-title hover italic shift (A3).
- **C2.6 M** — Checkbox border barely visible (B11); author column renders `—` dashes (data, fine).

### 3. /admin/edit/10029 — Post editor
- **C3.1 VA** — Right sidebar metabox tail ("Edit Snippet"/Rank Math tabs area) slides *under* the dock; pinned sidebar `#postbox-container-1.wp-pin` uses `top:32px` while adminbar is 33px (B17) → 1px overlap seam.
- **C3.2 P** — `#wp-content-editor-tools` toolbar row: `#f0f0f0`-derived bg hardcoded in cms-overrides (A7); "Add Media" button 40px vs Visual/Code tabs 29px on same flex row — center-aligned fine but the 11px size gap looks accidental.
- **C3.3 P** — Title input: `font-size 22.1px` (vendored 1.7em, fractional) with `padding:3px 8px` — text sits tight against a square-cornered (vendor) border; atrium never restyles `#title`. Fix: `#title{border-radius:var(--r-sm); padding:8px 12px; font-size:22px}`.
- **C3.4 P** — Publish box: "Update" (40px gradient) beside "Preview Changes" (40px paper) consistent height — OK; but `#major-publishing-actions` bg `--paper-2` with only `border-top` (no side padding issues) while `#delete-action` red `oklch(58% .18 25)` fine.
- **C3.5 M** — Language/Translations metabox: second radio row ("French") indented under flag icon — flag img + radio misaligned baseline (screenshot).
- **C3.6 M** — Rank Math score chip `82/100` inline in `.hndle` — bright green pill vs atrium palette; vendored markup, CSS token-able to `--teal`/`--ok`? Flag.

### 4. /admin/comments
- **C4.1 VA** — Persisted screen-options hide *all* data columns (author/comment/response/date all `.hidden`) → table renders as 1368px-wide glass card of 21 checkbox-only rows (measured). Massive dead card, no empty/hidden-columns hint. Fix direction (CSS-possible): `.wp-list-table .check-column{width:2.2em}` pins the column so at least it shrinks; a CSS-only "all hidden" notice isn't possible — flag for markup.
- **C4.2 M** — Dock badge "30" pending-count on Comments tile = good accent; no issue.

### 5. /admin/media
- **C5.1 P** — Toolbar packs 5 selects (Date/Language/Author/Type) + Filter button + view-switch icons + search + "9621 items" pager into one 32px row — selects at 30px vs search 32px vs Filter 32px: aligned via flex (cms-overrides `.tablenav.top`) — tight but OK; view-switch icons render as tiny faint dashicons (muted until hover) — acceptable.
- **C5.2 P** — Grid tiles have inconsistent thumbs: `attach-preview` portrait images letterboxed with white gaps (vendor `margin` centering) — `#att-grid .attachment` gets `--r-sm` radius + overflow hidden ✓; hover outline 2px accent ✓ (CSS present).
- **C5.3 M** — `spacebar` between "9621 items" text and pager buttons is `gap:6px`-ish default — cramped (screenshot: `» 1 of 482` text-tight to arrows).

### 6. /admin/menus
- **C6.1 VA** — Menu-name row: `Menu Name` label + 270px input + "Save Menu" (40px) — the input (32px) and button (40px) misaligned baseline, and the section header "Menu Structure" sits directly above with no visual grouping; left `#menu-settings-column` is empty (w=281,h=2) yet reserves 300px gutter → content column shifted right with dead space left.
- **C6.2 P** — Item bars 42px for parents vs 62px for nested items (measured) — nested bar's title wraps → taller pill; the `↕`-handle (`.menu-item-handle`) bar looks like a table row rather than a draggable card (postbox styling absent — items are `li.menu-item`, not `.postbox`). Fix: `li.menu-item .menu-item-bar{background:var(--paper);border-radius:var(--r-sm)}` token.
- **C6.3 M** — "create a new menu" link under header is `--accent` inline — fine; accordion left panel missing entirely (empty column, C6.1 root cause).

### 7. /admin/widgets
- **C7.1 VA** — `.widgets-holder-wrap` containers render `background:#fff; border-radius:0` (measured) — flat white vendor panels inside a fully-glassed skin; every other card in admin is `--glass`/`--r-lg`. Also `margin-bottom:20px` uniform but top padding missing → "Custom sidebars" group heading hugs the dock (screenshot: dock overlays its title/counter).
- **C7.2 P** — Column widths 411px vs 891px (measured) — asymmetric but intentional (widget list vs sidebars); right column ~35px gap too wide? `.widgets-liquid-right` uses vendor % widths — acceptable.
- **C7.3 P** — Sidebar widget titles 15px Space Grotesk (measured) vs postbox headers 13-14px — widget titles *larger* than panel headers; and `.widget-top` toggle arrow renders default WP chevron, no transition.
- **C7.4 M** — "Inactive Widgets" panel bottom hugs next panel at 20px but inner `.widgets-sortables` empty drop-zone shows 20px-high ghost band (measured h=20) — fine.

### 8. /admin/users
- **C8.1 P** — Single-user table: 6 filter selects all at same size aligned left, search right — fine; pager disabled arrows `«`/`›` render muted inside bordered buttons (32px) — consistent, but `1 of 1` text at 10px mono is the smallest text on any screen (B21).
- **C8.2 M** — "Add New User" pta blue again (A1); bulk-action row duplicated top+bottom correctly.
- **C8.3 M** — Username column shows email `admin@aucoeur-test.local` — data-driven, no avatar column; `—` dashes elsewhere consistent.

### 9. /admin/settings/general
- **C9.1 VA** — "Save Changes" button sits ~200px above dock but on shorter windows would collide; more importantly disabled-looking readonly fields (Site Title etc.) have *no* control chrome — text appears as plain labels (B10) → form reads as label soup with no affordance of being locked. (Verify: likely `readonly`/disabled markup — CSS should still paint `--paper-2` bg on `[disabled]`/`[readonly]`.)
- **C9.2 P** — Date/time format radios: 15px circles, labels flush left, but "Custom:" row has a 15px date-format input directly after a 280px date preview text — tiny input next to full-width preview (screenshot).
- **C9.3 M** — `.form-table th` 14px semibold ink vs `td` ink-2 — fine; `description` texts muted ok; `code` chips `--accent-deep` on `--paper-2` good.

### 10. /admin/login
- **C10.1 P** — Language switcher: `select` 54px tall, cropped descenders, globe icon `position:static` floating *above* the select row (measured `pos:static`), "Change" button beside at 32px → three misaligned pieces (screenshot). Fix: `.language-switcher form{display:flex;align-items:center;gap:8px}` + select height 32px.
- **C10.2 P** — "Remember Me" checkbox 1px border `--rule-2` ≈ invisible (B11); label sits 2px above checkbox baseline.
- **C10.3 M** — Card top whitespace: logo block ends at ~185px inside a 502px card → ~50px dead gap before "Username" label (h1 logo has `height:72px` + padding-top:78px even though no site-name text renders).
- **C10.4 M** — Login button uses `.submit .button-primary` full-width gradient — good; autofocus outline on username shows the 35%-opacity ring (A9 inconsistency).

### 11. /admin/tax/category + /admin/tax/region
- **C11.1 VA** — (region) single row ~2600px tall — `column-description` unclamped (B5); row content also overflows under the dock at the bottom.
- **C11.2 P** — Left "Add Región" column: "Parent Región" select 214px vs Name/Slug inputs 340px — ragged column edge (B12); "Add Región" button 40px vs inputs 32px (intentional).
- **C11.3 M** — Typo `Categorys can have a hierarchy` — template copy, not CSS, flag anyway.
- **C11.4 M** — Count column: number links render right-aligned mono blue (`#3858e9`? or accent) — consistent w/ posts count col; fine.
- **C11.5 M** — Search field top-right floats with `Search Categories` button — aligned; but the empty `.search-box` input placeholder is blank (no "Search categories…" placeholder text) — markup flag.

### 12. /admin/contact
- **C12.1 VA** — Empty table with no `no-items` row → big empty glass card (B4).
- **C12.2 P** — `Add New` pta blue (A1) — third instance.
- **C12.3 M** — Header row alone renders 5 labels with even spacing — looks like a complete table; add empty-state row or collapse card.

---

## Quick-win summary (highest ROI for the token pass)

1. **`.wrap a.page-title-action`** → join the atrium button group (kills the last visible WP blue).
2. **Global `a:focus-visible` + unify focus-ring token** (one outline, one offset).
3. **Pin `.check-column` width** + ship/clamp `column-description` (`line-clamp`).
4. **Empty-state rows** for contact/comments-all-hidden (markup flag; CSS can't conjure text).
5. **Tokenize cms-overrides literals** (`#fff`, `#f0f0f0`, `rgba(0,0,0,.4)` → `var(--paper)`/`var(--rule)`/`var(--shadow-*)`).
6. **`.widgets-holder-wrap`** → glass/`--r-lg` treatment to match `.postbox`.
7. **Hide subsubsub `|` separators**; de-dupe `.current` pills (template flag).
8. **Drop italic-on-hover** for row titles; add `transition:filter` to `.button-primary`.
9. **`.language-switcher` flex row** on login (fixes the floating globe + 54px select).
10. **adminbar 32 vs 33px** — `box-sizing` or pin-offset correction (1px seam on every pinned toolbar).

---

## Resolution pass (261030 — phase-02/03, commit d260cf4)

| Ref | Status |
|---|---|
| A1/B1/C2.1/C8.2/C12.2 pta blue | **fixed** — `.wp-admin .wrap a.page-title-action` joins the button system (verified: br 9px, paper bg) |
| A2/A9 focus rings | **fixed** — `--focus-ring` token; global `a:focus-visible`; checkbox/radio `:focus`→`:focus-visible` |
| A3 italic hover | **fixed** |
| A4/C2.3 `\|` pills | **fixed** — `ul.subsubsub{font-size:0}`, `li a{font-size:13px}` |
| A5 button-primary transition | **fixed** |
| A6 duplicate fallbacks | kept (intentional no-blur fallback pattern) |
| A7 cms-overrides literals | **fixed** — tokens with literal fallbacks |
| A8 MOTION pass misalign | **fixed** — block removed |
| A10 grain z-index | kept (pointer-events:none already; below flyouts) |
| B2 button heights | kept — `.button-large` 40px is deliberate hierarchy |
| B3 disabled pager | **fixed** — opacity .45 + default cursor |
| B4/C4.1/C12.1 empty states | **markup flag** — needs `tr.no-items` emission; CSS cannot conjure text |
| B5/C11.1 desc clamp | **fixed** — `-webkit-line-clamp:7` (2600px → ≤160px row, verified) |
| B6 tall rows | partially — `column-description` clamp helps tax; SEO cell left (data, acceptable) |
| B7 zebra-kill contrast | kept (deliberate flat + hover) |
| B8 check-column | **fixed** — 2.4em (1368px → 27px verified) |
| B9 row-actions | **fixed** — `--ink-2` |
| B10/B12 disabled | **fixed** — locked surface on `:disabled`/`[readonly]` |
| B11 checkbox | **fixed** — muted border + drawn check (login + admin) |
| B13 search pill variants | declined — responsive redesign is intentional |
| B14 notice chip | **fixed** — `vertical-align:-1px` |
| B16 dock overlap | declined — glass is readable by design; padding-bottom:120px reserves space |
| B17/C3.1 1px seam | **fixed** — adminbar `box-sizing:border-box; height:32px` (verified 32) |
| B18 flyout anim | **fixed** — `atrium-flyout` keyframes |
| B19 collapse focus | **fixed** |
| B20 footer gutter | **fixed** — 24px |
| B21 pager size | **fixed** — 12px mono, input 30px |
| B22 active tab | **fixed** — `--glass` |
| C1.1 postbox header 14px | declined — consistent across widgets, spec difference is invisible |
| C1.4 postbox gap | declined — card rhythm reads fine at both widths |
| C2.2 dual `.current` | **markup flag** — template emits it (posts.rs `on("","")` twice) |
| C3.2 toolbar row | partially — literals tokenized; size gap is WP parity |
| C3.3 title input | **fixed** — radius + padding |
| C3.4 publish box | ok as-is |
| C3.5 flag baseline | ok — flags are emoji, radios aligned (verified) |
| C3.6 RM chip | kept — vendored plugin accent |
| C5.x media | ok as-is |
| C6.1 menu-name baseline | **fixed** — `.menu-item-bar` cards; save row reads clean (screenshot) |
| C6.2 item bars | **fixed** |
| C7.1 widgets flat panels | **fixed** — glass + r-lg (verified) |
| C7.3 widget titles 15px | **fixed** — 13px |
| C9.x settings | **fixed** via B10 |
| C10.1 lang switcher | **fixed** — flex row (verified) |
| C10.3 login dead gap | kept — logo block layout |
| C11.2 col widths | declined — form widths are WP parity |
| C11.3/C11.5 typos/placeholder | **markup flags** — template copy |

Functional gate (phase-04): all actions green — create/save/quickedit/trash/restore/bulk/empty-trash,
comments approve↔pending cycle, menus locations save, widget save round-trip (sidebar preserved),
settings discussion save persisted, search/filters 200, zero console errors on 8 screens, 1024px
layout holds. Diff scope: 2 CSS files only.

## Code-review pass (post-commit, code-reviewer)

| Ref | Verdict |
|---|---|
| `#att-modal` lost `left:50%` during token swap | **CRITICAL — fixed** (modal rendered off-screen left; restored + deduped shadow; verified centered cx=720/1440) |
| `td.column-description` on mobile | **fixed** — clamp scoped `@media (min-width:783px)` so the card layout keeps `display:block;width:100%` |
| `.ui-sortable-placeholder` rule | dead CSS — widgets screen has no jQuery sortable init; harmless, kept as stub |
| 17 other named checks | OK — see agent review (pta specificity, focus rings, subsubsub, check-column, flyout keyframes, adminbar 32px, login switcher, no markup/JS in diff) |

## Second deep audit (code-reviewer, 15-item hunt)

| Ref | Verdict |
|---|---|
| vendored `checkbox/radio:focus` scheme-colored box-shadow halo | **fixed** — reset to `border-color:--muted; box-shadow:none`; same for text inputs/selects |
| `input#current-page-selector` dead rule | **fixed** — retargeted to emitted `.current-page` class |
| vendored `select:focus` + `.wp-tags-autocomplete` theme-color leak | **fixed** — box-shadow killed; autocomplete → `--accent-soft`/`--ink` |
| admin-color-* scheme strategy | **documented** — coral accent is the skin's brand; per-scheme remapping declined, leaks neutralized per-component |
| dead `.ui-sortable-placeholder` | kept as stub (no sortable init on widgets yet) |
| everything else (widget columns, menu bars, mask centering, flyout refire, readonly intent, check-column thead/tfoot, TinyMCE iframe vars, 783/782 boundary, subsubsub a11y) | OK / OK-with-note — see agent report |
