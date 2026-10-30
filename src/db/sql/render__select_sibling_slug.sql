SELECT id, lang, kind, slug FROM posts
WHERE slug=?1 AND lang<>?2 AND (?3='' OR kind=?3)
AND status='publish' LIMIT 1
