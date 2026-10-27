-- `page_on_front` → published post id. A draft page_on_front must never
-- render at `/` — every reader of this setting gates on publish.
SELECT CAST(s.value AS INT) FROM settings s
WHERE s.key='page_on_front'
  AND EXISTS(SELECT 1 FROM posts p WHERE p.id=CAST(s.value AS INT) AND p.status='publish')