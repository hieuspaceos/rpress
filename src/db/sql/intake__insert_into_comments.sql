-- No id: SQLite assigns via rowid — `MAX(id)+1` raced under concurrent
-- writers.
INSERT INTO comments(wp_id, post_id, parent_id, author, email, body, author_url, author_ip, status, created_at)
         VALUES(NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'approved', datetime('now'))
