//! Save path for the tourmaster editor plugin — port of
//! `tourmaster_save_tour_meta` + `page-option.php`'s single-meta split
//! (reference/tourmaster/tour/include/tour-option.php lines 1311–1381,
//! include/utility.php lines 779–878).

use crate::db::posts::PostForm;
use crate::php_serialize::{serialize, unserialize};
#[allow(unused_imports)]
use crate::sql;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde_json::{Map, Value};

/// `(payload_slug, meta_key)` pairs — from the PHP schema's
/// `'single' => '<meta-key>'` (payload keys are the *field slugs*, not
/// meta-minus-prefix).
pub(crate) const SINGLE_METAS: &[(&str, &str)] = &[
    ("thumbnail-link", "tourmaster-thumbnail-link"),
    ("thumbnail-video-url", "tourmaster-thumbnail-video-url"),
    ("extra-booking-info", "tourmaster-extra-booking-info"),
    ("contact-detail-fields", "tourmaster-contact-detail-fields"),
    ("enquiry-form-fields", "tourmaster-enquiry-form-fields"),
    ("enquiry-form-mail-content-admin", "tourmaster-enquiry-form-mail-content-admin"),
    ("enquiry-form-mail-content-customer", "tourmaster-enquiry-form-mail-content-customer"),
    ("book-in-advance", "tourmaster-book-in-advance"),
    ("tour-price-range", "tourmaster-tour-price-range"),
    ("minimum-people-per-booking", "tourmaster-min-people-per-booking"),
    ("maximum-people-per-booking", "tourmaster-max-people-per-booking"),
    ("maximum-people", "tourmaster-max-people"),
    ("carbon-copy-mail", "tourmaster-tour-cc-mail"),
    ("enable-payment-notification", "tourmaster-payment-notification"),
    ("enable-reminder-message", "tourmaster-reminder-message"),
];

fn set_meta(c: &Connection, id: i64, key: &str, value: &str) -> Result<()> {
    // PHP update_post_meta writes '' rows — keep parity with the import.
    c.execute(sql!("posts__delete_post_meta"), params![id, key])?;
    c.execute(sql!("posts__insert_post_meta"), params![id, key, value])?;
    Ok(())
}

fn del_meta(c: &Connection, id: i64, key: &str) -> Result<()> {
    c.execute(sql!("posts__delete_post_meta"), params![id, key])?;
    Ok(())
}

fn get_meta(c: &Connection, id: i64, key: &str) -> Option<String> {
    c.query_row(
        sql!("posts__select_meta_value_from"),
        params![id, key],
        |r| r.get(0),
    )
    .ok()
}

fn meta_to_json(raw: &str) -> Value {
    if raw.trim_start().starts_with('{') || raw.trim_start().starts_with('[') {
        serde_json::from_str(raw).unwrap_or(Value::Null)
    } else {
        unserialize(raw.as_bytes()).unwrap_or(Value::Null)
    }
}

/// Weekday index → PHP lowercase weekday name.
const WEEKDAYS: [&str; 7] = [
    "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
];

fn weekday_of(year: i64, month: u32, day: u32) -> u32 {
    // Sakamoto's algorithm; 0 = Sunday .. 6 = Saturday
    const T: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    ((y + y / 4 - y / 100 + y / 400 + T[(month - 1) as usize] + day as i64) % 7) as u32
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn valid_date(s: &str) -> bool {
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 3 {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        p[0].parse::<i64>(),
        p[1].parse::<u32>(),
        p[2].parse::<u32>(),
    ) else {
        return false;
    };
    p[0].len() == 4 && (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)
}

/// Port of `tourmaster_get_tour_dates` for one `date-price` entry.
fn expand_entry(entry: &Map<String, Value>, timing: &str, out: &mut Vec<String>) {
    if timing == "single" {
        if let Some(d) = entry.get("date").and_then(|v| v.as_str()) {
            if !d.is_empty() {
                out.push(d.to_string());
            }
        }
        return;
    }
    let days: Vec<String> = entry
        .get("day")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let months: Vec<u32> = entry
        .get("month")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str()?.parse().ok()).collect())
        .unwrap_or_default();
    let years: Vec<i64> = entry
        .get("year")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str()?.parse().ok()).collect())
        .unwrap_or_default();
    if !days.is_empty() && !months.is_empty() && !years.is_empty() {
        for &y in &years {
            for &m in &months {
                for wd in &days {
                    let Some(wd_idx) = WEEKDAYS.iter().position(|d| d == wd) else {
                        continue;
                    };
                    // PHP: start at {year}-{month}-1, if weekday matches take
                    // it, then walk "next {day}" while still in the month.
                    for d in 1..=days_in_month(y, m) {
                        // PHP weekday index: Sunday=0..Saturday=6; map to
                        // WEEKDAYS (monday=0).
                        let php_wd = weekday_of(y, m, d) as usize; // 0=Sun
                        let ours = (php_wd + 6) % 7; // 0=Mon
                        if ours == wd_idx {
                            out.push(format!("{y:04}-{m:02}-{d:02}"));
                        }
                    }
                }
            }
        }
    }
    if let Some(extra) = entry.get("extra-date").and_then(|v| v.as_str()) {
        for d in extra.split(',').map(str::trim) {
            if valid_date(d) && !out.contains(&d.to_string()) {
                out.push(d.to_string());
            }
        }
        out.sort();
    }
    if let Some(excl) = entry.get("exclude-extra-date").and_then(|v| v.as_str()) {
        let ex: Vec<&str> = excl.split(',').map(str::trim).collect();
        out.retain(|d| !ex.contains(&d.as_str()));
    }
}

/// Port of `tourmaster_filter_tour_date` (book-in-advance months +
/// past-date drop, using local date — good enough for admin sync).
fn filter_dates(dates: &[String], months_ahead: &str, today: &str) -> Vec<String> {
    let Ok(months) = months_ahead.trim().parse::<i64>() else {
        return dates
            .iter()
            .filter(|d| d.as_str() >= today)
            .cloned()
            .collect();
    };
    if months == 0 {
        return dates
            .iter()
            .filter(|d| d.as_str() >= today)
            .cloned()
            .collect();
    }
    // end_time = first day of this month + (months+1) months, exclusive.
    let p: Vec<&str> = today.split('-').collect();
    if p.len() != 3 {
        return dates.to_vec();
    }
    let (Ok(y), Ok(m), _) = (p[0].parse::<i64>(), p[1].parse::<i64>(), p[2].parse::<i64>()) else {
        return dates.to_vec();
    };
    let total = y * 12 + (m - 1) + months + 1;
    let (ey, em) = (total / 12, total % 12 + 1);
    let end = format!("{ey:04}-{em:02}-01");
    dates
        .iter()
        .filter(|d| d.as_str() >= today && d.as_str() < end.as_str())
        .cloned()
        .collect()
}

fn today_str() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // civil-from-days (UTC); sites run on VN server TZ but day-granularity
    // availability is close enough for the editor sync.
    let days = (t / 86400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

pub(crate) fn save(c: &Connection, id: i64, f: &PostForm) -> Result<()> {
    // PHP: absent POST field → reuse existing meta (e.g. non-tour forms).
    let raw = f
        .extra
        .get("tourmaster-tour-option")
        .map(|s| s.as_str())
        .unwrap_or("");
    let dict: Map<String, Value> = if raw.trim().is_empty() {
        match get_meta(c, id, "tourmaster-tour-option") {
            Some(b) => meta_to_json(&b).as_object().cloned().unwrap_or_default(),
            None => Map::new(),
        }
    } else {
        let mut d: Map<String, Value> = serde_json::from_str::<serde_json::Value>(raw)
            .context("tourmaster-tour-option JSON")?
            .as_object()
            .cloned()
            .unwrap_or_default();
        // split singles out of the blob — payload keys are field slugs
        for (slug, meta) in SINGLE_METAS {
            if let Some(v) = d.remove(*slug) {
                let s = match &v {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                set_meta(c, id, meta, &s)?;
            }
        }
        let blob = serialize(&Value::Object(d.clone()));
        c.execute(sql!("posts__delete_post_meta"), params![id, "tourmaster-tour-option"])?;
        c.execute(
            sql!("posts__insert_post_meta"),
            params![id, "tourmaster-tour-option", String::from_utf8_lossy(&blob)],
        )?;
        d
    };

    // ---- derived metas (tourmaster_save_tour_meta) ----
    let timing = dict
        .get("tour-timing-method")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let mut dates: Vec<String> = Vec::new();
    if let Some(Value::Array(entries)) = dict.get("date-price") {
        if !timing.is_empty() {
            for e in entries {
                if let Some(m) = e.as_object() {
                    expand_entry(m, timing, &mut dates);
                }
            }
        }
    }
    dates.sort();
    dates.dedup();
    if !dates.is_empty() {
        set_meta(c, id, "tourmaster-tour-date", &dates.join(","))?;
        let book = dict
            .get("book-in-advance")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let avail = filter_dates(&dates, book, &today_str());
        if avail.is_empty() {
            del_meta(c, id, "tourmaster-tour-date-avail")?;
        } else {
            set_meta(c, id, "tourmaster-tour-date-avail", &avail.join(","))?;
        }
    } else {
        del_meta(c, id, "tourmaster-tour-date")?;
        del_meta(c, id, "tourmaster-tour-date-avail")?;
    }

    match dict.get("tour-type").and_then(|v| v.as_str()) {
        Some("single") => set_meta(c, id, "tourmaster-tour-duration", "1")?,
        Some(_) => {
            let d = dict
                .get("multiple-duration")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if d.is_empty() {
                del_meta(c, id, "tourmaster-tour-duration")?;
            } else {
                set_meta(c, id, "tourmaster-tour-duration", d)?;
            }
        }
        None => {}
    }

    let price_txt = dict
        .get("tour-price-text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let disc_txt = dict
        .get("tour-price-discount-text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !disc_txt.is_empty() {
        set_meta(c, id, "tourmaster-tour-price", disc_txt)?;
        set_meta(c, id, "tourmaster-tour-discount", "true")?;
    } else if !price_txt.is_empty() {
        set_meta(c, id, "tourmaster-tour-price", price_txt)?;
        set_meta(c, id, "tourmaster-tour-discount", "false")?;
    } else {
        del_meta(c, id, "tourmaster-tour-price")?;
        del_meta(c, id, "tourmaster-tour-discount")?;
    }

    // ---- tours table sync (same source as promote_tours) ----
    let duration: Option<i64> = get_meta(c, id, "tourmaster-tour-duration")
        .and_then(|s| s.parse().ok());
    let price_range = get_meta(c, id, "tourmaster-tour-price-range");
    let max_people: Option<i64> = get_meta(c, id, "tourmaster-max-people")
        .and_then(|s| s.parse().ok());
    c.execute(
        sql!("posts__insert_into_tours_19"),
        params![
            id,
            duration,
            price_range.filter(|s| !s.is_empty()),
            max_people,
            if dates.is_empty() { None } else { Some(dates.join(",")) },
        ],
    )?;
    Ok(())
}
