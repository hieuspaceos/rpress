fn main() {
    let s = std::fs::read("/Users/hieuspace/Desktop/CODE/pet/rpress/work/aucoeur-test/dump.sql").unwrap();
    let s = String::from_utf8_lossy(&s);
    let i = s.find("'a:25:{i:0;a:2:{i:0;s:7").unwrap();
    // find terminating '\',' or "',"  — value ends at unescaped '\''
    let start = i+1;
    let bytes = s.as_bytes();
    let mut end = start;
    while end < bytes.len() {
        if bytes[end] == b'\'' && bytes[end-1] != b'\\' { break; }
        end += 1;
    }
    let raw = &s[start..end];
    // SQL unescape \' and \\
    let un = raw.replace("\\'","'").replace("\\\\","\\").replace("\\r\\n","\r\n").replace("\\n","\n");
    match rpress::php_serialize::unserialize(un.as_bytes()) {
        Ok(v) => println!("OK type {:?}", std::mem::discriminant(&v)),
        Err(e) => println!("ERR {e}"),
    }
}
