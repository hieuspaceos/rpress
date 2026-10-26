SELECT p.id, p.slug, p.title, COALESCE(p.excerpt,''), p.featured_media_id
             FROM posts p JOIN post_terms pt ON pt.post_id=p.id
             WHERE pt.taxonomy=?1 COLLATE NOCASE AND pt.slug=?2 AND p.kind='tour' AND p.status='publish' AND p.lang=?3
             ORDER BY p.published_at DESC