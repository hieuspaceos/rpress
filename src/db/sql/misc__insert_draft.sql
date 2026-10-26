INSERT INTO posts(id, wp_id, slug, lang, title, body_json, status, modified_at, kind)
VALUES(?1, ?1, ?2, 'fr', ?3, ?4, 'draft', datetime('now'), 'post')
