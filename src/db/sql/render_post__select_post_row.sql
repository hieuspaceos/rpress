SELECT COALESCE(published_at,''), featured_media_id, seo_title, seo_desc
FROM posts WHERE id=?1 LIMIT 1
