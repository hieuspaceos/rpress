SELECT m2.post_id, p.lang, p.kind, p.slug FROM post_translation_members m
JOIN post_translation_members m2 ON m2.group_id=m.group_id AND m2.lang=?2
JOIN posts p ON p.id=m2.post_id AND p.status='publish'
WHERE m.post_id=?1 LIMIT 1
