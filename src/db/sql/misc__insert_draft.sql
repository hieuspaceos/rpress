-- No id: `last_insert_rowid()` reads it back — `MAX(id)+1` raced under
-- concurrent writers.
INSERT INTO posts(slug, lang, title, body_json, status, modified_at, kind)
VALUES(?1, 'fr', ?2, ?3, 'draft', datetime('now'), 'post')
