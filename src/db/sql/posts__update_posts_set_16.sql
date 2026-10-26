UPDATE posts SET title=?1, slug=?2, lang=?3, excerpt=?4, seo_title=?5, seo_desc=?6,
             status=?7, featured_media_id=?8, body_json=?9, body_html=?11,
             published_at=COALESCE(?10, published_at),
             modified_at=datetime('now') WHERE id=?12
