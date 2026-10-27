-- No id: caller checks execute()==1 then reads last_insert_rowid() —
-- `MAX(id)+1` raced under concurrent writers.
INSERT INTO comments(wp_id, post_id, parent_id, author, email, author_url, author_ip, body, status, created_at)
SELECT NULL, c.post_id, c.id, ?2, ?3, '', '', ?4, 'approved', datetime('now')
FROM comments c WHERE c.id=?1
