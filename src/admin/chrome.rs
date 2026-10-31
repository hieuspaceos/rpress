//! The WP-chrome frame every admin screen renders inside: `html_admin` /
//! `html_page` wrappers, the admin menu (`chrome_start`/`menu_li`), and the
//! closing chrome (`chrome_end`).
use crate::db as crud;
use crate::plugins;
use axum::response::Html;

use super::users;
use super::util::esc;
use super::Sess;

pub(crate) fn html_page(s: &Sess, title: &str, body: &str, htmx: bool) -> Html<String> {
    html_admin(s, title, body, htmx, "index-php")
}

pub(crate) fn html_admin(s: &Sess, title: &str, body: &str, htmx: bool, screen: &str) -> Html<String> {
    let src = if htmx {
        r#"<script src="/admin/vendor/htmx.min.js"></script>"#
    } else {
        ""
    };
    Html(format!(
        r#"<!doctype html><html class="wp-toolbar" lang="en"><head><meta charset="utf-8"><title>{title}</title>
<link rel="stylesheet" href="/admin/wp/dashicons.min.css">
<link rel="stylesheet" href="/admin/wp/admin-bar.min.css">
<link rel="stylesheet" href="/admin/wp/common.min.css">
<link rel="stylesheet" href="/admin/wp/forms.min.css">
<link rel="stylesheet" href="/admin/wp/admin-menu.min.css">
<link rel="stylesheet" href="/admin/wp/dashboard.min.css">
<link rel="stylesheet" href="/admin/wp/list-tables.min.css">
<link rel="stylesheet" href="/admin/wp/edit.min.css">
<link rel="stylesheet" href="/admin/wp/editor.min.css">

<link rel="stylesheet" href="/admin/wp/media.min.css">
<link rel="stylesheet" href="/admin/wp/nav-menus.min.css">
<link rel="stylesheet" href="/admin/wp/l10n.min.css">
<link rel="stylesheet" href="/admin/wp/admin-schemes.min.css">
<link rel="stylesheet" href="/admin/wp/wp-tooltip.min.css">
<link rel="stylesheet" href="/admin/wp/buttons.min.css">
<link rel="stylesheet" href="/admin/wp/colors.min.css">
<link rel="stylesheet" href="/admin/wp/cms-overrides.css">
<link rel="stylesheet" href="/admin/wp/atrium.css">
<script>var ajaxurl='/admin/'; var pagenow='{screen}';</script>
<script>var wp=window.wp||{{}};wp.i18n={{__:function(s){{return s;}},_x:function(s){{return s;}},sprintf:function(f){{var a=[].slice.call(arguments,1);var i=0;return f.replace(/%[sd]/g,function(){{return String(a[i++]);}});}}}};</script>
<script src="/admin/wp/jquery.min.js"></script>
<script src="/admin/wp/hoverintent-js.min.js"></script>
<script src="/admin/wp/hoverIntent.min.js"></script>
<script src="/admin/wp/common.min.js"></script>
<script src="/admin/wp/admin-bar.min.js"></script>
<script src="/admin/wp/svg-painter.js"></script>
{src}</head>
<body class="wp-admin wp-core-ui js {screen} auto-fold admin-bar {scheme} svg">{body}
{end}</body></html>"#,
        scheme = users::color_class(s),
        end = chrome_end(&s)
    ))
}


pub(crate) fn chrome(active: &str, s: &Sess) -> String {
    chrome_start(active, s)
}

pub(crate) fn menu_li<A: AsRef<str>, B: AsRef<str>>(
    id: &str,
    href: &str,
    label: &str,
    icon: &str,
    active: &str,
    sub: &[(A, B)],
) -> String {
    menu_li_badge(id, href, label, icon, active, sub, 0)
}

/// `menu_li` + WP pending-count bubble (`awaiting-mod`) when `badge > 0`.
pub(crate) fn menu_li_badge<A: AsRef<str>, B: AsRef<str>>(
    id: &str,
    href: &str,
    label: &str,
    icon: &str,
    active: &str,
    sub: &[(A, B)],
    badge: i64,
) -> String {
    let on = active == id;
    let cur = if on {
        "wp-has-submenu wp-has-current-submenu wp-menu-open menu-top current"
    } else if sub.is_empty() {
        "wp-not-current-submenu menu-top"
    } else {
        "wp-has-submenu wp-not-current-submenu menu-top"
    };
    let aclass = if on {
        "wp-has-current-submenu wp-menu-open current"
    } else if sub.is_empty() {
        ""
    } else {
        "wp-has-submenu"
    };
    let bubble = if badge > 0 {
        format!(" <span class=\"awaiting-mod count-{badge}\"><span class=\"pending-count\" aria-hidden=\"true\">{badge}</span><span class=\"comments-in-moderation-text screen-reader-text\">{badge} pending</span></span>")
    } else {
        String::new()
    };
    let mut out = format!(
        r#"<li id="menu-{id}" class="{cur}"><a href="{href}" class="menu-top {aclass}"><div class="wp-menu-image dashicons-before {icon}"><br></div><div class="wp-menu-name">{label}{bubble}</div></a>"#
    );
    if !sub.is_empty() {
        out.push_str(&format!(
            r#"<ul class="wp-submenu wp-submenu-wrap"><li class="wp-submenu-head" aria-hidden="true">{label}</li>"#
        ));
        for (i, (h, l)) in sub.iter().enumerate() {
            let first = if i == 0 { " wp-first-item" } else { "" };
            out.push_str(&format!(r#"<li class="{first}"><a href="{}">{}</a></li>"#, h.as_ref(), l.as_ref()));
        }
        out.push_str("</ul>");
    }
    out.push_str("</li>");
    out
}

pub(crate) fn chrome_start(active: &str, s: &Sess) -> String {
    let site_name = esc(&s.site_name);
    let email = esc(&s.email);
    let mut html = String::from(
        r###"<div id="wpwrap">
<div id="adminmenumain" role="navigation" aria-label="Main menu">
<a class="screen-reader-shortcut" href="#wpbody-content">Skip to main content</a>
<a class="screen-reader-shortcut" href="#wp-toolbar">Skip to toolbar</a>
<div id="adminmenuback"></div>
<div id="adminmenuwrap">
<ul id="adminmenu">
"###,
    );

    html.push_str(&menu_li("dashboard", "/admin", "Dashboard", "dashicons-dashboard", active, &[] as &[(&str, &str)]));
    html.push_str(&menu_li(
        "posts",
        "/admin/posts",
        "Posts",
        "dashicons-admin-post",
        active,
        &[("/admin/posts", "All Posts"), ("/admin/new/post", "Add Post"), ("/admin/tax/category", "Categories"), ("/admin/tax/post_tag", "Tags")],
    ));
    html.push_str(&menu_li(
        "media",
        "/admin/media",
        "Media",
        "dashicons-admin-media",
        active,
        &[("/admin/media", "Library"), ("/admin/media/new", "Add New")],
    ));
    html.push_str(&menu_li(
        "pages",
        "/admin/pages",
        "Pages",
        "dashicons-admin-page",
        active,
        &[("/admin/pages", "All Pages"), ("/admin/new/page", "Add Page")],
    ));
    let pending_comments = crud::comments::counts(&s.db).map(|c| c.pending).unwrap_or(0);
    html.push_str(&menu_li_badge(
        "comments",
        "/admin/comments",
        "Comments",
        "dashicons-admin-comments",
        active,
        &[] as &[(&str, &str)],
        pending_comments,
    ));
    html.push_str(&menu_li(
        "reviews",
        "/admin/reviews",
        "Site Reviews",
        "dashicons-star-filled",
        active,
        &[] as &[(&str, &str)],
    ));
    // Registered plugin kinds get one generic menu entry each (adapter
    // may override the submenu via `PluginAdapter::submenu`).
    // Plugin kinds with no imported content get no dock tile (e.g. babe
    // `to_book` on a site that never used it) — the section + list routes
    // still resolve, the tile just stops advertising an empty screen.
    let kind_has_posts = |kind: &str| -> bool {
        rusqlite::Connection::open(&s.db)
            .ok()
            .and_then(|c| c.query_row(sql!("posts__count_by_kind"), [kind], |r| r.get::<_, i64>(0)).ok())
            .map(|n| n > 0)
            .unwrap_or(true)
    };
    for def in plugins::kind_defs() {
        if def.menu_icon.is_empty() || matches!(def.kind, "post" | "page") || !kind_has_posts(def.kind) {
            continue;
        }
        html.push_str(&menu_li(
            def.section,
            &format!("/admin/{}", def.section),
            def.list_title,
            def.menu_icon,
            active,
            &plugins::submenu_for(&s.db, def),
        ));
    }

    html.push_str(&menu_li(
        "contact",
        "/admin/contact",
        "Contact",
        "dashicons-email",
        active,
        &[
            ("/admin/contact", "Forms"),
            ("/admin/contact/inbox", "Submissions"),
        ],
    ));
    // Appearance: Menus for every role; Widgets is `edit_theme_options`
    // parity → admin-only, matching the /admin/widgets* guards.
    let appearance_sub: Vec<(&str, &str)> = if s.role == "admin" {
        vec![("/admin/menus", "Menus"), ("/admin/widgets", "Widgets")]
    } else {
        vec![("/admin/menus", "Menus")]
    };
    html.push_str(&menu_li(
        "menus",
        "/admin/menus",
        "Appearance",
        "dashicons-admin-appearance",
        active,
        &appearance_sub,
    ));
    if s.role == "admin" {
        html.push_str(&menu_li(
            "plugins",
            "/admin/plugins",
            "Plugins",
            "dashicons-admin-plugins",
            active,
            &[("/admin/plugins", "Installed Plugins")],
        ));
    }
    if s.role == "admin" {
        html.push_str(&menu_li(
            "users",
            "/admin/users",
            "Users",
            "dashicons-admin-users",
            active,
            &[("/admin/users", "All Users"), ("/admin/user-new", "Add User")],
        ));
    }
    if s.role == "admin" {
        html.push_str(&menu_li(
            "settings",
            "/admin/settings/general",
            "Settings",
            "dashicons-admin-settings",
            active,
            &[
                ("/admin/settings/general", "General"),
                ("/admin/settings/writing", "Writing"),
                ("/admin/settings/reading", "Reading"),
                ("/admin/settings/discussion", "Discussion"),
                ("/admin/settings/media", "Media"),
                ("/admin/settings/permalink", "Permalinks"),
                ("/admin/settings/privacy", "Privacy"),
            ],
        ));
    }
    if s.role == "admin" {
        html.push_str(&menu_li(
            "tools",
            "/admin/tools/backup",
            "Tools",
            "dashicons-admin-tools",
            active,
            &[("/admin/tools/backup", "Backup")],
        ));
    }

    html.push_str(
        r#"<li id="collapse-menu" class="hide-if-no-js"><button type="button" id="collapse-button" aria-label="Collapse Main menu" aria-expanded="true"><span class="collapse-button-icon" aria-hidden="true"></span><span class="collapse-button-label">Collapse Menu</span></button></li>
</ul>
</div>
</div>
<div id="wpcontent">
<div id="wpadminbar" class="nojq nojs" role="navigation">
<div class="quicklinks" id="wp-toolbar">
<ul id="wp-admin-bar-root-default" class="ab-top-menu">
<li id="wp-admin-bar-site-name" class="menupop"><a class="ab-item" href="/">"#,);
    html.push_str(&site_name);
    html.push_str(r#"</a>
<div class="ab-sub-wrapper"><ul class="ab-submenu"><li id="wp-admin-bar-view-site"><a class="ab-item" href="/">View Site</a></li></ul></div></li>
<li id="wp-admin-bar-new-content" class="menupop"><a class="ab-item" href="/admin/new/post">New</a>
<div class="ab-sub-wrapper"><ul class="ab-submenu">
<li><a class="ab-item" href="/admin/new/post">Post</a></li>
<li><a class="ab-item" href="/admin/new/page">Page</a></li>
<li><a class="ab-item" href="/admin/new/tour">Tour</a></li>
</ul></div></li>
</ul>
<ul id="wp-admin-bar-top-secondary" class="ab-top-secondary ab-top-menu">
<li id="wp-admin-bar-my-account" class="menupop"><a class="ab-item" href="/admin/profile">Howdy, "#,
    );
    html.push_str(&email);
    html.push_str(&format!(
        r#"</a>
<div class="ab-sub-wrapper"><ul class="ab-submenu"><li id="wp-admin-bar-logout"><form method="post" action="/api/logout" style="margin:0"><input type="hidden" name="csrf" value="{csrf}"><button type="submit" class="ab-item" style="background:none;border:0;cursor:pointer;width:100%;text-align:left">Log Out</button></form></li></ul></div></li>
</ul>
</div>
</div>
<div id="wpbody" role="main">
<div id="wpbody-content">
"#,
        csrf = esc(&s.csrf)
    ));
    html
}


pub(crate) fn chrome_end(s: &Sess) -> String {
    let site_name = esc(&s.site_name);
    format!(r#"<div class="clear"></div>
</div><!-- wpbody-content -->
<div class="clear"></div>
</div><!-- wpbody -->
<div class="clear"></div>
</div><!-- wpcontent -->
<div id="wpfooter" role="contentinfo">
<p id="footer-left" class="alignleft">{site_name}</p>
<p id="footer-upgrade" class="alignright">Version 7.1.3 chrome</p>
</div>
<script src="/admin/wp/atrium.js" defer></script>
</div><!-- wpwrap -->"#)
}
