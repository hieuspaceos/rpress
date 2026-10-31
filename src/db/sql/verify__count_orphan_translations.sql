SELECT COUNT(*) FROM post_translation_members m
WHERE NOT EXISTS(SELECT 1 FROM posts p WHERE p.id=m.post_id)