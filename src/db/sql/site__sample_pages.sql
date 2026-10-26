-- check phase sample: first published pages/posts/tours — no per-site hardcoding.
SELECT id, slug, lang, kind FROM posts
WHERE status='publish' AND kind IN ('page','post','tour')
ORDER BY id LIMIT 6;
