-- No id: caller reads last_insert_rowid() then stamps slug `review-{id}` —
-- `MAX(id)+1` raced under concurrent writers.
INSERT INTO posts(slug, lang, title, body_json, status, published_at, modified_at, kind)
                         VALUES('review-pending','fr',?1,?2,'publish',datetime('now'),datetime('now'),'site-review')
