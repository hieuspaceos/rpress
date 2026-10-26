INSERT INTO posts(id, wp_id, slug, lang, title, body_json, body_html, excerpt, status, published_at, modified_at, seo_title, seo_desc, featured_media_id, kind)
             VALUES(?1,?1,?2,?3,?4,?5,?13,?6,?7,COALESCE(?12,datetime('now')),datetime('now'),?8,?9,?10,?11)
