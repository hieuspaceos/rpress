//! Shared WP-list-table chrome: `ul.subsubsub` view links, `div.tablenav`
//! (bulk actions + pagination), pager, and `.row-actions` spans.
//! WP parity: views with zero rows are hidden; per_page is fixed at 20.
use crate::admin::{esc, urlenc};
use std::collections::HashMap;

pub(crate) const PER_PAGE: i64 = 20;

/// One `subsubsub` view link, e.g. `Pending <span class=count>(3)</span>`.
pub(crate) struct View {
    pub key: &'static str,
    pub label: &'static str,
    pub count: i64,
}

/// `All (42) | Pending (3) | Spam (0)` — zero-count views are omitted (WP/
/// Flamingo behavior). `base` must already contain `?` or get `?` appended.
pub(crate) fn views_bar(base: &str, param: &str, current: &str, views: &[View], extra: &HashMap<String, String>) -> String {
    let mut out = String::from("<ul class=\"subsubsub\">");
    let mut first = true;
    for v in views {
        if v.count == 0 && v.key != "all" && v.key != "inbox" {
            continue;
        }
        if !first {
            out.push_str(" | ");
        }
        first = false;
        let cur = if current == v.key { " class=\"current\" aria-current=\"page\"" } else { "" };
        out.push_str(&format!(
            "<li class=\"{}\"><a href=\"{}{}={}{}{}\"{}>{} <span class=\"count\">({})</span></a></li>",
            v.key,
            base,
            param,
            v.key,
            qs(extra),
            if v.key == "all" || v.key == "inbox" { "" } else { "" },
            cur,
            v.label,
            v.count
        ));
    }
    out.push_str("</ul>");
    out
}

/// Extra query params appended to view links (`&k=v` escaped).
fn qs(extra: &HashMap<String, String>) -> String {
    let mut s = String::new();
    for (k, v) in extra {
        if !v.is_empty() {
            s.push_str(&format!("&{}={}", k, urlenc(v)));
        }
    }
    s
}

/// Public wrapper — sortable-header links outside this module need the
/// same `&k=v` encoding.
pub(crate) fn qs_pub(extra: &HashMap<String, String>) -> String {
    qs(extra)
}

/// `?paged=` pager → `LIMIT, OFFSET` + `.tablenav-pages` markup.
pub(crate) struct Pager {
    pub page: i64,
    pub total: i64,
    /// Rows per page — `PER_PAGE` unless a screen options pane overrides it.
    pub per_page: i64,
}

impl Pager {
    pub fn new(q: &HashMap<String, String>, total: i64) -> Self {
        let page = q.get("paged").and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
        Pager { page, total, per_page: PER_PAGE }.clamp()
    }
    /// Pager when the page number arrives already parsed (posts list takes
    /// `page`/`per_page` as args rather than the raw query map).
    pub(crate) fn at(page: i64, total: i64, per_page: i64) -> Self {
        Pager { page, total, per_page: per_page.max(1) }.clamp()
    }
    /// Screen Options override: `per_page` comes from user prefs.
    pub fn with_per_page(mut self, per: i64) -> Self {
        self.per_page = per.max(1);
        self.clamp()
    }
    /// Clamp `page` into [1, pages] — an over-range `?paged=999` otherwise
    /// renders an empty list with no way back (audit: pager clamp).
    fn clamp(mut self) -> Self {
        self.page = self.page.clamp(1, self.pages().max(1));
        self
    }
    pub fn pages(&self) -> i64 {
        (self.total + self.per_page - 1) / self.per_page
    }
    pub fn limit(&self) -> i64 {
        self.per_page
    }
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
    /// `<div class=tablenav-pages>…` — WP first/prev/next/last with disabled
    /// spans, `X of Y pages`, `N items`.
    pub fn nav(&self, base: &str, extra: &HashMap<String, String>) -> String {
        let pages = self.pages().max(1);
        let mut out = format!(
            "<div class=\"tablenav-pages\"><span class=\"displaying-num\">{} item{}</span><span class=\"pagination-links\">",
            self.total,
            if self.total == 1 { "" } else { "s" }
        );
        let link = |p: i64| format!("{}?paged={}{}", base, p, qs(extra));
        if self.page <= 1 {
            out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">«</span>");
            out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">‹</span>");
        } else {
            out.push_str(&format!("<a class=\"first-page button\" href=\"{}\"><span aria-hidden=\"true\">«</span></a>", link(1)));
            out.push_str(&format!("<a class=\"prev-page button\" href=\"{}\"><span aria-hidden=\"true\">‹</span></a>", link(self.page - 1)));
        }
        out.push_str(&format!(
            "<span class=\"paging-input\"><label class=\"screen-reader-text\">Current Page</label><input class=\"current-page\" type=\"text\" name=\"paged\" value=\"{}\" size=\"1\"> of <span class=\"total-pages\">{}</span></span>",
            self.page, pages
        ));
        if self.page >= pages {
            out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">›</span>");
            out.push_str("<span class=\"tablenav-pages-navspan button disabled\" aria-hidden=\"true\">»</span>");
        } else {
            out.push_str(&format!("<a class=\"next-page button\" href=\"{}\"><span aria-hidden=\"true\">›</span></a>", link(self.page + 1)));
            out.push_str(&format!("<a class=\"last-page button\" href=\"{}\"><span aria-hidden=\"true\">»</span></a>", link(pages)));
        }
        out.push_str("</span></div>");
        out
    }
}

/// `<div class=tablenav top|bottom>`: bulk-action select + Apply + pager
/// (top only shows pager; WP mirrors bulk on both). Bottom select is named
/// `action2` — serde_urlencoded last-wins would let an untouched bottom
/// select shadow a chosen top action (WP does the same).
pub(crate) fn tablenav(top: bool, actions: &[(&str, &str)], pager: Option<&Pager>, base: &str, extra: &HashMap<String, String>) -> String {
    let mut out = format!("<div class=\"tablenav {}\">", if top { "top" } else { "bottom" });
    if !actions.is_empty() {
        let name = if top { "action" } else { "action2" };
        out.push_str(&format!("<div class=\"actions bulkactions\"><label class=\"screen-reader-text\">Select bulk action</label><select name=\"{name}\">"));
        out.push_str("<option value=\"-1\">Bulk actions</option>");
        for (v, label) in actions {
            out.push_str(&format!("<option value=\"{}\">{}</option>", v, label));
        }
        out.push_str("</select><input type=\"submit\" class=\"button action\" value=\"Apply\"></div>");
    }
    if top {
        if let Some(p) = pager {
            out.push_str(&p.nav(base, extra));
        }
    }
    out.push_str("<br class=\"clear\"></div>");
    out
}

/// `<thead>` row from (class, label, sortable_href_or_empty) tuples.
pub(crate) fn thead(cols: &[(&str, &str, &str)]) -> String {
    let mut out = String::from("<thead><tr><td class=\"manage-column column-cb check-column\"><input type=\"checkbox\" id=\"cb-select-all-1\"></td>");
    for (class, label, sort) in cols {
        if sort.is_empty() {
            out.push_str(&format!("<th class=\"manage-column column-{class}\">{label}</th>", class = class, label = label));
        } else {
            out.push_str(&format!(
                "<th class=\"manage-column column-{class} sortable desc\"><a href=\"{sort}\"><span>{label}</span><span class=\"sorting-indicators\"></span></a></th>",
                class = class,
                sort = sort,
                label = label
            ));
        }
    }
    out.push_str("</tr></thead>");
    out
}

/// Empty-state row spanning all columns.
pub(crate) fn empty_row(cols: usize, msg: &str) -> String {
    format!("<tr class=\"no-items\"><td class=\"colspanchange\" colspan=\"{}\">{}</td></tr>", cols, esc(msg))
}
/// Redirect target preserved across POSTs (WP `_wp_http_referer` minimal).
/// Same-origin only: `//host` protocol-relative URLs are rejected too.
pub(crate) fn back_or<'a>(f: &'a HashMap<String, String>, default: &'a str) -> &'a str {
    f.get("back")
        .map(|s| s.as_str())
        .filter(|s| s.starts_with('/') && !s.starts_with("//"))
        .unwrap_or(default)
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn views_hide_zero_counts() {
        let v = views_bar("/admin/comments?", "status", "pending", &[
            View { key: "all", label: "All", count: 10 },
            View { key: "pending", label: "Pending", count: 3 },
            View { key: "spam", label: "Spam", count: 0 },
        ], &HashMap::new());
        assert!(v.contains("Pending"));
        assert!(!v.contains("Spam"));
    }
    #[test]
    fn pager_offsets() {
        let mut q = HashMap::new();
        q.insert("paged".into(), "2".into());
        let p = Pager::new(&q, 45);
        assert_eq!(p.offset(), 20);
        assert_eq!(p.pages(), 3);
        assert!(p.nav("/x", &HashMap::new()).contains("last-page"));
    }
}
