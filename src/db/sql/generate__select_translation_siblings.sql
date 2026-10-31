SELECT m2.post_id FROM post_translation_members m
JOIN post_translation_members m2 ON m2.group_id=m.group_id
WHERE m.post_id=?1 AND m2.post_id!=?1