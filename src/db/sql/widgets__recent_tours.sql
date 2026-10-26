SELECT id FROM posts WHERE kind='tour' AND status='publish' AND lang=?1 AND slug NOT LIKE '%-copy' ORDER BY published_at DESC LIMIT ?2
