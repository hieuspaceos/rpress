SELECT p.id, COALESCE(p.published_at,''),
(SELECT meta_value FROM post_meta WHERE post_id=p.id AND meta_key='_thumbnail_id' LIMIT 1)
FROM posts p JOIN post_terms pt ON pt.post_id=p.id
WHERE p.kind='ml-slide' AND p.status='publish'
AND pt.taxonomy='ml-slider' AND pt.slug=?1
ORDER BY p.menu_order
