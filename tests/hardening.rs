use rpress::db::{served_upload_mime, UPLOAD_LIMIT_BYTES, UPLOAD_LIMIT_DISPLAY};
use rpress::generate::atomic_write;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rpress-{name}-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn scriptable_legacy_uploads_force_download() {
    for name in ["x.svg", "x.xml", "x.html", "x.htm", "x.js", "x.mjs", "x.unknown"] {
        let (mime, attachment) = served_upload_mime(name);
        assert_eq!(mime, "application/octet-stream", "{name}");
        assert!(attachment, "{name}");
    }
}

#[test]
fn safe_uploads_keep_inline_media_types() {
    for (name, want) in [
        ("x.jpg", "image/jpeg"),
        ("x.jpeg", "image/jpeg"),
        ("x.png", "image/png"),
        ("x.webp", "image/webp"),
        ("x.gif", "image/gif"),
        ("x.pdf", "application/pdf"),
        ("x.woff2", "font/woff2"),
        // WP writes theme-generated stylesheets into uploads/; they must
        // serve inline or the public site loses its custom CSS.
        ("x.css", "text/css"),
    ] {
        let (mime, attachment) = served_upload_mime(name);
        assert_eq!(mime, want, "{name}");
        assert!(!attachment, "{name}");
    }
}

#[test]
fn upload_limit_contract_is_25mb() {
    assert_eq!(UPLOAD_LIMIT_BYTES, 25 * 1024 * 1024);
    assert_eq!(UPLOAD_LIMIT_DISPLAY, "25 MB");
}

#[test]
fn atomic_write_replaces_existing_file() {
    let dir = temp_dir("atomic");
    let path = dir.join("nested/index.html");
    atomic_write(&path, "old").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "old");
    atomic_write(&path, "new").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "new");
    let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("tmp-"))
        .collect();
    assert!(leftovers.is_empty());
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn static_reads_reject_symlink_escape() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir("symlink");
    let root = dir.join("root");
    let outside = dir.join("secret.txt");
    fs::create_dir_all(&root).unwrap();
    fs::write(&outside, "secret").unwrap();
    symlink(&outside, root.join("leak.txt")).unwrap();

    assert_eq!(rpress::admin::safe_read_under(&root, "leak.txt"), None);
    let _ = fs::remove_dir_all(dir);
}
