SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id, COALESCE(p.published_at,'') FROM posts p
