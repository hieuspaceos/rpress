CREATE TABLE post_meta(post_id INTEGER, meta_key TEXT, meta_value TEXT,
    PRIMARY KEY(post_id, meta_key, meta_value));
-- 'zzz' has the SMALLER rowid but the LARGER meta_value: the
-- covering index serves 'aaa' to LIMIT-1 readers, so 'aaa'
-- must be the keeper (not MIN(rowid)).
INSERT INTO post_meta VALUES(7,'a','zzz'),(7,'a','aaa'),(7,'b','x'),(8,'a','y');