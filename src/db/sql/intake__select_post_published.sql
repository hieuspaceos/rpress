SELECT COALESCE(published_at,'') FROM posts WHERE id=?1 AND status='publish' LIMIT 1
