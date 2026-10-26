SELECT id, COALESCE(title,'') FROM posts
WHERE kind=?1 AND status='publish' AND (?2='' OR lang=?2)
ORDER BY published_at DESC LIMIT 15
