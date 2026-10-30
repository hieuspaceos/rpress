//! CF7-style tag parser — subset only. Parses `[type* name opt:val ...]`
//! tags inside form markup. Used by:
//!   - `forms_editor.rs` to validate markup on save (reject unknown types,
//!     missing names, unclosed brackets, `[name]` mail refs that don't exist)
//!   - `intake.rs` to enforce required fields + type-check on submission.
//!
//! Supported types: text, email, url, tel, number, date, textarea, select,
//! checkbox, radio, acceptance, quiz, file, submit.

use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)] // opts/at retained for error reporting completeness
pub struct Tag {
    /// `text` / `email` / `textarea` / ...
    pub type_: String,
    /// Field name (`your-name`, `rating`, ...). `submit` has an empty name.
    pub name: String,
    /// Whether `*` was present.
    pub required: bool,
    /// Raw options: `min:1`, `max:5`, `id:foo`, `class:bar`, quoted
    /// strings for `select`/labels.
    pub opts: Vec<String>,
    /// Byte offset in source (for error reporting).
    pub at: usize,
}

const KNOWN: &[&str] = &[
    "text", "email", "url", "tel", "number", "date", "textarea", "select",
    "checkbox", "radio", "acceptance", "quiz", "file", "submit",
    // CF7 extension plugins: country-phone-field (phonetext/countrytext),
    // turnstile/recaptcha markers (self-closing, no name),
    // hidden/dynamictext fields.
    "phonetext", "countrytext", "cf7sr-turnstile", "turnstile",
    "hidden", "dynamictext",
];

/// Parse all `[...]` tags in markup. Returns `Err(msg)` on the first
/// malformed or unknown tag. Bare `[` and `]` inside text are tolerated
/// (skipped) — CF7 is forgiving there too.
pub fn parse(markup: &str) -> Result<Vec<Tag>, String> {
    let b = markup.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'[' {
            i += 1;
            continue;
        }
        // find matching ']' (no nesting in CF7)
        let mut j = i + 1;
        while j < b.len() && b[j] != b']' {
            // '[' before ']' → the first '[' was text
            if b[j] == b'[' {
                break;
            }
            j += 1;
        }
        if j >= b.len() || b[j] != b']' {
            i += 1; // treat as text
            continue;
        }
        let inner = &markup[i + 1..j];
        if let Some(tag) = parse_tag(inner).map_err(|e| format!("{e} (at byte {i})"))? {
            out.push(Tag { at: i, ..tag });
        }
        i = j + 1;
    }
    Ok(out)
}

fn parse_tag(inner: &str) -> Result<Option<Tag>, String> {
    let s = inner.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let mut it = s.split_whitespace();
    let mut typ = it.next().unwrap_or("");
    if typ.is_empty() {
        return Ok(None);
    }
    // CF7 types may carry a trailing `*` — e.g. `text*`
    let required = typ.ends_with('*');
    if required {
        typ = &typ[..typ.len() - 1];
    }
    if !KNOWN.contains(&typ) {
        return Err(format!("unknown tag type '{typ}'"));
    }
    // Collect opts. `submit` may have just a label; others need a name.
    let mut opts: Vec<String> = Vec::new();
    let mut name = String::new();
    let mut first = true;
    for tok in it {
        if first && !tok.contains(':') && !tok.starts_with('"') {
            name = tok.to_string();
            first = false;
            continue;
        }
        first = false;
        opts.push(tok.to_string());
    }
    // `submit` and captcha markers carry no field name.
    if !matches!(typ, "submit" | "cf7sr-turnstile" | "turnstile") && name.is_empty() {
        return Err(format!("tag '{typ}' missing name"));
    }
    Ok(Some(Tag {
        type_: typ.to_string(),
        name,
        required,
        opts,
        at: 0,
    }))
}

/// Names of all non-submit fields — used to validate `[name]` mail refs.
pub fn field_names(tags: &[Tag]) -> Vec<&str> {
    tags.iter()
        .filter(|t| t.type_ != "submit")
        .map(|t| t.name.as_str())
        .collect()
}

/// Validate markup + mail templates together. Returns the first error.
pub fn validate(markup: &str, mail_to: &str, mail_from: &str, mail_subject: &str, mail_body: &str) -> Result<(), String> {
    let tags = parse(markup)?;
    if tags.iter().all(|t| t.type_ != "submit") {
        return Err("form has no [submit] tag".into());
    }
    let names: Vec<&str> = field_names(&tags);
    let check_refs = |tpl: &str, what: &str| -> Result<(), String> {
        for m in tpl.match_indices('[') {
            if let Some(end) = tpl[m.0..].find(']') {
                let name = &tpl[m.0 + 1..m.0 + end];
                if !name.is_empty() && !names.contains(&name) {
                    return Err(format!("{what} references [{name}] which is not a field"));
                }
            }
        }
        Ok(())
    };
    check_refs(mail_subject, "mail subject")?;
    check_refs(mail_body, "mail body")?;
    check_refs(mail_to, "mail to")?;
    check_refs(mail_from, "mail from")?;
    Ok(())
}

/// Required fields by name — used at intake time.
#[allow(dead_code)] // intake-validation parity; retained as library surface
pub fn required_names(tags: &[Tag]) -> Vec<&str> {
    tags.iter()
        .filter(|t| t.required && !t.name.is_empty())
        .map(|t| t.name.as_str())
        .collect()
}

/// Light type-check for a submitted value — returns an error key like
/// `email_invalid` when the value looks wrong. Only checks `email`, `url`,
/// `tel`, `number`, `date` (others accept anything).
pub fn check_value(type_: &str, v: &str) -> Option<&'static str> {
    if v.is_empty() {
        return None; // required-ness checked elsewhere
    }
    match type_ {
        "email" => {
            if v.contains('@') && v.split('@').nth(1).map(|d| d.contains('.')).unwrap_or(false) {
                None
            } else {
                Some("email_invalid")
            }
        }
        "url" => {
            if v.starts_with("http://") || v.starts_with("https://") {
                None
            } else {
                Some("url_invalid")
            }
        }
        "number" => {
            if v.parse::<f64>().is_ok() {
                None
            } else {
                Some("number_invalid")
            }
        }
        "date" => {
            let ok = v.len() == 10
                && v.chars().nth(4) == Some('-')
                && v.chars().nth(7) == Some('-');
            if ok {
                None
            } else {
                Some("date_invalid")
            }
        }
        "tel" => {
            if v.chars().all(|c| c.is_ascii_digit() || "+-() ".contains(c)) {
                None
            } else {
                Some("tel_invalid")
            }
        }
        _ => None,
    }
}

/// Substitute `[name]` placeholders in a template with submitted values.
#[allow(dead_code)] // CF7 mail-body templating parity; retained as library surface
pub fn substitute(tpl: &str, values: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(tpl.len());
    let mut rest = tpl;
    while let Some(i) = rest.find('[') {
        out.push_str(&rest[..i]);
        if let Some(j) = rest[i..].find(']') {
            let name = &rest[i + 1..i + j];
            if let Some(v) = values.get(name) {
                out.push_str(v);
            } else {
                out.push_str(&rest[i..i + j + 1]);
            }
            rest = &rest[i + j + 1..];
        } else {
            out.push_str(&rest[i..]);
            break;
        }
    }
    out.push_str(rest);
    out
}
/// Render a form's `markup` to HTML controls — the renderer half of the
/// CF7 shortcode (`templates/elements/cf7_*.html` were per-site captures;
/// this builds inputs from the `forms` row so every form is correct).
///
/// `[cf7sr-turnstile]`/`[turnstile]` emit nothing (Cloudflare script is
/// dropped like other third-party JS). Quoted opts carry labels/values —
/// the same "`-free strings are values" convention CF7 uses.
pub fn form_html(markup: &str, slug: &str, site_url: &str) -> String {
    let esc = |s: &str| s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");
    let mut inner = String::with_capacity(markup.len() + 1024);
    let mut rest = markup;
    while let Some(i) = rest.find('[') {
        inner.push_str(&rest[..i]);
        match rest[i..].find(']') {
            Some(j) => {
                let inner_s = rest[i + 1..i + j].trim();
                match crate::forms_markup::parse_tag(inner_s).ok().flatten() {
                    Some(t) => inner.push_str(&tag_html(&t, &esc)),
                    None => inner.push_str(&rest[i..i + j + 1]),
                }
                rest = &rest[i + j + 1..];
            }
            None => {
                inner.push_str(&rest[i..]);
                rest = "";
            }
        }
    }
    inner.push_str(rest);
    format!(
        "<div class=\"wpcf7 no-js\"><form action=\"{site_url}/form/{}\" method=\"post\" class=\"wpcf7-form init\" novalidate=\"novalidate\">\
         <input type=\"hidden\" name=\"_form\" value=\"{}\" />\
         <input type=\"text\" name=\"_gotcha\" value=\"\" style=\"display:none\" tabindex=\"-1\" autocomplete=\"off\" />\
         {}</form></div>",
        esc(slug), esc(slug), inner
    )
}

/// One tag → its control HTML. Mirrors the plugin's output shape:
/// `<span class="wpcf7-form-control-wrap" data-name="…"><input …/></span>`
/// — the site CSS hooks exactly these classes.
fn tag_html(t: &Tag, esc: &dyn Fn(&str) -> String) -> String {
    // Quoted tokens = literal values (labels/options/placeholders). The
    // parser split opts on whitespace — fragments like `"Nom` `et`
    // `prénom` `*"` reassemble only when we join then re-extract "…".
    let joined = t.opts.join(" ");
    let vals: Vec<String> = joined.split('"').enumerate()
        .filter(|(i, _)| i % 2 == 1).map(|(_, s)| s.to_string()).collect();
    let optv = |k: &str| -> String {
        t.opts.iter().find_map(|o| o.strip_prefix(&format!("{k}:")).map(str::to_string))
            .unwrap_or_default()
    };
    let mut cls = format!("wpcf7-form-control wpcf7-{}", t.type_);
    if t.required { cls.push_str(" wpcf7-validates-as-required"); }
    let id_attr = { let v = optv("id"); if v.is_empty() { String::new() } else { format!(" id=\"{}\"", esc(&v)) } };
    let req = if t.required { " aria-required=\"true\" required" } else { "" };
    let wrap = |inner: &str| format!("<span class=\"wpcf7-form-control-wrap\" data-name=\"{}\">{inner}</span>", esc(&t.name));
    match t.type_.as_str() {
        "submit" => format!("<input class=\"wpcf7-form-control wpcf7-submit has-spinner\" type=\"submit\" value=\"{}\"{id_attr} />", esc(vals.first().map(|s| s.as_str()).unwrap_or("Submit"))),
        "textarea" => {
            let ph = if vals.is_empty() { String::new() } else { format!(" placeholder=\"{}\"", esc(&vals[0])) };
            wrap(&format!("<textarea name=\"{}\" cols=\"40\" rows=\"10\" class=\"wpcf7-textarea {cls}\"{req}{id_attr}{ph}></textarea>", esc(&t.name)))
        }
        "select" => {
            let mut o = String::new();
            for v in &vals { o.push_str(&format!("<option value=\"{0}\">{0}</option>", esc(v))); }
            wrap(&format!("<select name=\"{}\" class=\"wpcf7-select {cls}\"{req}{id_attr}>{o}</select>", esc(&t.name)))
        }
        "checkbox" | "radio" => {
            let ty = &t.type_;
            let mut items = String::new();
            for (ix, v) in vals.iter().enumerate() {
                let first = if ix == 0 { " first" } else { "" };
                items.push_str(&format!(
                    "<span class=\"wpcf7-list-item{first}\"><label><input type=\"{ty}\" name=\"{}[]\" value=\"{}\" /><span class=\"wpcf7-list-item-label\">{}</span></label></span>",
                    esc(&t.name), esc(v), esc(v)));
            }
            wrap(&format!("<span class=\"{cls}\">{items}</span>"))
        }
        "acceptance" => wrap(&format!(
            "<span class=\"wpcf7-acceptance\"><span class=\"wpcf7-list-item\"><label><input type=\"checkbox\" name=\"{}\" value=\"1\"{req} /><span class=\"wpcf7-list-item-label\">{}</span></label></span></span>",
            esc(&t.name), esc(vals.first().map(|s| s.as_str()).unwrap_or("")))),
        "quiz" => {
            let q = vals.first().map(|v| v.split('|').next().unwrap_or(v).to_string()).unwrap_or_default();
            wrap(&format!("<label>{q}</label><input type=\"text\" name=\"{}\" size=\"40\" class=\"wpcf7-quiz {cls}\"{req} autocomplete=\"off\" />", esc(&t.name)))
        }
        "phonetext" => {
            // country-phone-field CF7: text input + hidden country-code
            // field (the +CC selected flag). Origin emits both.
            let dv = vals.first().map(|s| s.as_str()).unwrap_or("");
            wrap(&format!(
                "<input size=\"40\" class=\"wpcf7-phonetext wpcf7-text {cls}\" value=\"{}\" type=\"tel\" name=\"{}\"{req} />\
                 <input type=\"hidden\" name=\"{}-country-code\" class=\"wpcf7-phonetext-country-code\" />",
                esc(dv), esc(&t.name), esc(&t.name)))
        }
        "countrytext" => {
            let cur = vals.first().map(|s| s.as_str()).unwrap_or("");
            let mut o = String::new();
            for c in COUNTRIES {
                let sel = if *c == cur { " selected" } else { "" };
                o.push_str(&format!("<option value=\"{0}\"{sel}>{0}</option>", esc(c)));
            }
            wrap(&format!("<select name=\"{}\" class=\"wpcf7-countrytext wpcf7-select {cls}\"{req}{id_attr}>{o}</select>", esc(&t.name)))
        }
        "hidden" | "dynamictext" => format!("<input type=\"hidden\" name=\"{}\" value=\"{}\" />", esc(&t.name), esc(vals.first().map(|s| s.as_str()).unwrap_or(""))),
        "file" => wrap(&format!("<input type=\"file\" name=\"{}\" class=\"wpcf7-file {cls}\"{req} />", esc(&t.name))),
        "cf7sr-turnstile" | "turnstile" => String::new(),
        _ => {
            let ty = match t.type_.as_str() {
                "email" => "email", "url" => "url", "tel" => "tel",
                "number" => "number", "date" => "date", _ => "text",
            };
            let ph = if vals.is_empty() { String::new() } else { format!(" placeholder=\"{}\"", esc(&vals[0])) };
            let min = optv("min"); let min_attr = if min.is_empty() { String::new() } else { format!(" min=\"{}\"", esc(&min)) };
            let max = optv("max"); let max_attr = if max.is_empty() { String::new() } else { format!(" max=\"{}\"", esc(&max)) };
            let cls2 = if t.type_ == "text" { format!("{cls} wpcf7-text") } else { cls };
            wrap(&format!("<input size=\"40\" class=\"{cls2}\" type=\"{ty}\" name=\"{}\"{req}{id_attr}{ph}{min_attr}{max_attr} />", esc(&t.name)))
        }
    }
}

/// `countrytext` country list — the plugin's own ordering. Rendered as a
/// plain <select>: a searchable flag widget needs the origin's JS bundle
/// which we deliberately don't ship.
const COUNTRIES: &[&str] = &[
    "Afghanistan","Albania","Algeria","Andorra","Angola","Antigua and Barbuda","Argentina","Armenia","Australia","Austria","Azerbaijan",
    "Bahamas","Bahrain","Bangladesh","Barbados","Belarus","Belgium","Belize","Benin","Bhutan","Bolivia","Bosnia and Herzegovina","Botswana",
    "Brazil","Brunei","Bulgaria","Burkina Faso","Burundi","Cabo Verde","Cambodia","Cameroon","Canada","Central African Republic","Chad","Chile",
    "China","Colombia","Comoros","Congo","Costa Rica","Croatia","Cuba","Cyprus","Czechia","Democratic Republic of the Congo","Denmark",
    "Djibouti","Dominica","Dominican Republic","Ecuador","Egypt","El Salvador","Equatorial Guinea","Eritrea","Estonia","Eswatini","Ethiopia",
    "Fiji","Finland","France","Gabon","Gambia","Georgia","Germany","Ghana","Greece","Grenada","Guatemala","Guinea","Guinea-Bissau","Guyana",
    "Haiti","Honduras","Hungary","Iceland","India","Indonesia","Iran","Iraq","Ireland","Israel","Italy","Ivory Coast","Jamaica","Japan",
    "Jordan","Kazakhstan","Kenya","Kiribati","Kuwait","Kyrgyzstan","Laos","Latvia","Lebanon","Lesotho","Liberia","Libya","Liechtenstein",
    "Lithuania","Luxembourg","Madagascar","Malawi","Malaysia","Maldives","Mali","Malta","Marshall Islands","Mauritania","Mauritius","Mexico",
    "Micronesia","Moldova","Monaco","Mongolia","Montenegro","Morocco","Mozambique","Myanmar","Namibia","Nauru","Nepal","Netherlands",
    "New Zealand","Nicaragua","Niger","Nigeria","North Korea","North Macedonia","Norway","Oman","Pakistan","Palau","Panama","Papua New Guinea",
    "Paraguay","Peru","Philippines","Poland","Portugal","Qatar","Romania","Russia","Rwanda","Saint Kitts and Nevis","Saint Lucia",
    "Saint Vincent and the Grenadines","Samoa","San Marino","Sao Tome and Principe","Saudi Arabia","Senegal","Serbia","Seychelles",
    "Sierra Leone","Singapore","Slovakia","Slovenia","Solomon Islands","Somalia","South Africa","South Korea","South Sudan","Spain",
    "Sri Lanka","Sudan","Suriname","Sweden","Switzerland","Syria","Taiwan","Tajikistan","Tanzania","Thailand","Timor-Leste","Togo","Tonga",
    "Trinidad and Tobago","Tunisia","Turkey","Turkmenistan","Tuvalu","Uganda","Ukraine","United Arab Emirates","United Kingdom",
    "United States","Uruguay","Uzbekistan","Vanuatu","Vatican City","Venezuela","Vietnam","Yemen","Zambia","Zimbabwe",
];
