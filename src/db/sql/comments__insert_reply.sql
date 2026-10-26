INSERT INTO comments(id, wp_id, post_id, parent_id, author, email, author_url, author_ip, body, status, created_at)
SELECT ?1, NULL, c.post_id, c.id, ?3, ?4, '', '', ?5, 'approved', datetime('now')
FROM comments c WHERE c.id=?2
