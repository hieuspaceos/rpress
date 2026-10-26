SELECT slug FROM posts
WHERE kind=?1 AND lang=?2 AND status='publish' AND published_at > ?3
ORDER BY published_at ASC, id ASC LIMIT 1
