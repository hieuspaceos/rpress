#[test]
fn cat_seq() {
    for q in ["kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat[]=a:b&cat[]=c:d&tags=z",
              "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat=a:b&cat=c:d&tags=z",
              "kind=post&title=x&slug=y&lang=fr&status=draft&terms=&cat[]=a:b&tags=z"] {
        match serde_urlencoded::from_str::<pressrs::db::posts::PostForm>(q) {
            Ok(f) => println!("OK {:?} cat={:?}", q, f.cat),
            Err(e) => println!("ERR {:?} -> {}", q, e),
        }
    }
}
