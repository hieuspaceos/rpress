use pressrs::php_serialize::{serialize, unserialize};
use serde_json::Value;

#[test]
fn polylang_description() {
    let blob = include_str!("samples/polylang.txt");
    let v = unserialize(blob.trim().as_bytes()).unwrap();
    let m = v.as_object().unwrap();
    assert_eq!(m["locale"], "fr_FR");
    assert_eq!(m["rtl"], 0);
    assert_eq!(m["flag_code"], "fr");
    assert_eq!(m["fallbacks"], Value::Array(vec![]));
}

#[test]
fn tourmaster_tour_option() {
    let blob = include_str!("samples/tourmaster.txt");
    let v = unserialize(blob.trim().as_bytes()).unwrap();
    let m = v.as_object().unwrap();
    assert!(m.contains_key("header-image"));
    assert_eq!(m.len(), 78);
}

#[test]
fn tourmaster_roundtrip() {
    let blob = include_str!("samples/tourmaster.txt");
    let v = unserialize(blob.trim().as_bytes()).unwrap();
    let back = unserialize(&serialize(&v)).unwrap();
    assert_eq!(v, back);
}

#[test]
fn scalars_roundtrip() {
    for v in [
        Value::Null,
        Value::Bool(true),
        Value::from(42i64),
        Value::from(1.5f64),
        Value::String("héllo\n".into()),
        serde_json::json!({"1": "x", "k": [1, 2]}),
    ] {
        assert_eq!(unserialize(&serialize(&v)).unwrap(), v);
    }
}

#[test]
fn gdlr_tree() {
    let blob = include_str!("samples/gdlr.txt");
    let v = unserialize(blob.trim().as_bytes()).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 11);
    let first = arr[0].as_object().unwrap();
    assert_eq!(first["template"], "wrapper");
    assert_eq!(first["type"], "background");
    assert!(first["value"].as_object().unwrap().len() > 50);
}
