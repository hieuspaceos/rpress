SELECT modified_at, featured_media_id, COALESCE(excerpt,''), COALESCE(published_at,'')
FROM posts WHERE id=?1 LIMIT 1
