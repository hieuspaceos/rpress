//! sqldump lexer coverage: mysqldump / phpMyAdmin statement variants and
//! gzip transport. Each dump is a byte-string constant written to a temp
//! file, then streamed through `sqldump::rows`.

use rpress::sqldump::{rows, Val};
use std::io::Write;
use std::path::PathBuf;

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rpress-sqldump-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("dump.sql")
}

fn collect(path: &std::path::Path, wanted: &[&str]) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    rows(path, wanted, |t, vals| {
        out.push((
            t.to_string(),
            vals.iter().map(|v| v.to_string_lossy().unwrap_or("<NULL>".into())).collect(),
        ));
        Ok(())
    })
    .unwrap();
    out
}

fn write(path: &std::path::Path, bytes: &[u8]) {
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn unquoted_insert_table_name() {
    let p = tmp("unquoted");
    write(&p, b"INSERT INTO wp_posts VALUES (1,'a');\n");
    let got = collect(&p, &["wp_posts"]);
    assert_eq!(got, vec![("wp_posts".into(), vec!["1".into(), "a".into()])]);
}

#[test]
fn insert_ignore_and_qualified_name() {
    let p = tmp("qualified");
    write(
        &p,
        b"INSERT IGNORE INTO `mydb`.`wp_terms` VALUES (1,'n','s');\n",
    );
    let got = collect(&p, &["wp_terms"]);
    assert_eq!(got, vec![("wp_terms".into(), vec!["1".into(), "n".into(), "s".into()])]);
}

#[test]
fn column_list_remapped_to_create_order() {
    let p = tmp("colmap");
    write(
        &p,
        b"CREATE TABLE `t1` (\n  `id` bigint NOT NULL,\n  `name` varchar(20),\n  `price` decimal(10,2),\n  PRIMARY KEY (`id`)\n) ENGINE=InnoDB;\n\
         INSERT INTO `t1` (`price`,`id`) VALUES (9,7);\n",
    );
    let got = collect(&p, &["t1"]);
    // price→col3, id→col1, missing `name` → NULL
    assert_eq!(got, vec![("t1".into(), vec!["7".into(), "<NULL>".into(), "9".into()])]);
}

#[test]
fn column_list_unknown_column_bails() {
    let p = tmp("colbad");
    write(
        &p,
        b"CREATE TABLE `t1` (`id` bigint, `name` varchar(20));\n\
         INSERT INTO `t1` (`id`,`bogus`) VALUES (1,2);\n",
    );
    let err = rows(&p, &["t1"], |_, _| Ok(())).unwrap_err();
    assert!(err.to_string().contains("bogus"), "unexpected error: {err}");
}

#[test]
fn column_list_without_create_table_bails() {
    let p = tmp("colnone");
    write(&p, b"INSERT INTO `t1` (`id`,`x`) VALUES (1,2);\n");
    let err = rows(&p, &["t1"], |_, _| Ok(())).unwrap_err();
    assert!(err.to_string().contains("no CREATE TABLE"), "unexpected error: {err}");
}

#[test]
fn charset_and_national_introducers() {
    let p = tmp("introducers");
    write(
        &p,
        b"INSERT INTO `t1` VALUES (N'ab',_utf8mb4'cd',_binary'ef');\n",
    );
    let got = collect(&p, &["t1"]);
    assert_eq!(got, vec![("t1".into(), vec!["ab".into(), "cd".into(), "ef".into()])]);
}

#[test]
fn latin1_introducer_transcodes() {
    let p = tmp("latin1");
    // _latin1's\xe9' — é in latin1 is 0xe9
    write(&p, b"INSERT INTO `t1` VALUES (_latin1's\xe9');\n");
    let mut out = Vec::new();
    rows(&p, &["t1"], |_, vals| {
        if let Val::Str(b) = &vals[0] {
            out.push(b.clone());
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(out, vec![vec![b's', 0xC3, 0xA9]]); // utf-8 'sé'
}

#[test]
fn backspace_formfeed_escapes() {
    let p = tmp("escapes");
    write(&p, b"INSERT INTO `t1` VALUES ('a\\bb\\fc\\Zd\\'e');\n");
    let mut out = Vec::new();
    rows(&p, &["t1"], |_, vals| {
        if let Val::Str(b) = &vals[0] {
            out.push(b.clone());
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(out, vec![vec![b'a', 0x08, b'b', 0x0C, b'c', 0x1A, b'd', b'\'', b'e']]);
}

#[test]
fn multiline_statement_with_semicolon_in_string() {
    let p = tmp("multiline");
    // line 1 ends with ';' inside an unterminated string — an
    // endswith(';') completeness check would truncate the statement.
    write(
        &p,
        b"INSERT INTO `t1` VALUES (1,'hello;\nworld'),(2,'done');\n",
    );
    let got = collect(&p, &["t1"]);
    assert_eq!(
        got,
        vec![
            ("t1".into(), vec!["1".into(), "hello;\nworld".into()]),
            ("t1".into(), vec!["2".into(), "done".into()]),
        ]
    );
}

#[test]
fn case_insensitive_keywords() {
    let p = tmp("case");
    write(&p, b"insert into wp_posts values (1,'x');\n");
    let got = collect(&p, &["wp_posts"]);
    assert_eq!(got.len(), 1);
}

#[test]
fn gzipped_dump_transparent() {
    let p = tmp("gzip");
    let sql = b"INSERT INTO `wp_terms` VALUES (1,'a','b'),(2,'c','d');\n";
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(sql).unwrap();
    std::fs::write(&p, enc.finish().unwrap()).unwrap();
    let got = collect(&p, &["wp_terms"]);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].1, ["1", "a", "b"].map(String::from));
}

#[test]
fn null_and_empty_and_doubled_quotes() {
    let p = tmp("nulls");
    write(&p, b"INSERT INTO `t1` VALUES (NULL,'','it''s','ok');\n");
    let got = collect(&p, &["t1"]);
    assert_eq!(
        got,
        vec![("t1".into(), vec!["<NULL>".into(), "".into(), "it's".into(), "ok".into()])]
    );
}

#[test]
fn replace_into_bails() {
    let p = tmp("replace");
    write(&p, b"REPLACE INTO `wp_posts` VALUES (1,'x');\n");
    let err = rows(&p, &["wp_posts"], |_, _| Ok(())).unwrap_err();
    assert!(err.to_string().contains("unsupported"), "unexpected error: {err}");
}

#[test]
fn insert_variants_bail_loudly() {
    // DELAYED/LOW_PRIORITY modifiers and INSERT … SELECT carry rows we
    // can't parse — never skip them silently.
    for (name, stmt) in [
        ("delayed", b"INSERT DELAYED INTO `wp_posts` VALUES (1,'x');\n" as &[u8]),
        ("select", b"INSERT wp_posts SELECT * FROM other;\n"),
        ("lowprio", b"INSERT LOW_PRIORITY INTO `wp_posts` VALUES (1,'x');\n"),
    ] {
        let p = tmp(name);
        write(&p, stmt);
        let err = rows(&p, &["wp_posts"], |_, _| Ok(())).unwrap_err();
        assert!(err.to_string().contains("unsupported"), "no bail for {name}: {err}");
    }
}
