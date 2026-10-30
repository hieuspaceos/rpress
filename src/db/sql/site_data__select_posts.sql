SELECT id, slug, title, COALESCE(excerpt,''), featured_media_id,
COALESCE(published_at,'') FROM posts
WHERE kind='post' AND lang=?1 AND status='publish'
ORDER BY published_at DESC
