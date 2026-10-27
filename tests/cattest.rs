/// `PostForm.cat` decoding contract (audit L — this test was print-only):
/// the checkbox list joins selections with \u{1f} inside a single `cat`
/// field; `de_cat` splits it back into slugs.
#[test]
fn cat_seq() {
    // Single cat field carrying US-separated slug:name pairs — the wire
    // format the post form actually emits.
    let f = serde_urlencoded::from_str::<rpress::db::posts::PostForm>(
        "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat=a%3Ab%1Fc%3Ad&tags=z",
    )
    .expect("US-joined cat parses");
    assert_eq!(f.cat, vec!["a:b".to_string(), "c:d".to_string()]);

    // PHP-style `cat[]` keys don't exist in our forms; they fall into the
    // `extra` flatten map, leaving `cat` empty rather than erroring.
    let f = serde_urlencoded::from_str::<rpress::db::posts::PostForm>(
        "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat[]=a:b&cat[]=c:d&tags=z",
    )
    .expect("cat[] form parses");
    assert!(f.cat.is_empty());
    assert_eq!(f.extra.get("cat[]").map(|s| s.as_str()), Some("c:d"));

    // Repeated bare `cat` keys are a duplicate field — rejected, not a
    // silent last-wins collapse.
    let r = serde_urlencoded::from_str::<rpress::db::posts::PostForm>(
        "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat=a:b&cat=c:d&tags=z",
    );
    assert!(r.is_err(), "duplicate cat keys must error, not collapse");
}
