-- No id: runtime insert path; SQLite assigns via rowid (see note in
-- intake::submit_comment — MAX(id)+1 raced under concurrent writers).
INSERT INTO comments(wp_id, post_id, parent_id, author, email, body, author_url, author_ip, status, created_at)
         VALUES(NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
