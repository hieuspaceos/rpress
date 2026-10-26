use rusqlite::{Connection, params};
fn main() {
    let conn = Connection::open("rpress.db").unwrap();
    let q = "SELECT title FROM posts WHERE slug=?1 AND (?2='' OR lang=?2) AND (?3='' OR kind=?3) AND status='publish' LIMIT 1";
    let r: Result<String,_> = conn.query_row(q, params!["montagnes-ethnie-du-nord-du-vietnam","", "tour"], |r| r.get(0));
    println!("{:?}", r);
}
