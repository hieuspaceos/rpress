-- No id: `last_insert_rowid()` reads it back — `MAX(id)+1` raced under
-- concurrent writers. wp_id stays NULL (same convention as clone_post).
INSERT INTO posts(slug, lang, title, body_json, body_html, excerpt, status, published_at, modified_at, seo_title, seo_desc, featured_media_id, kind)
             VALUES(?1,?2,?3,?4,?12,?5,?6,COALESCE(?11,datetime('now')),datetime('now'),?7,?8,?9,?10)
