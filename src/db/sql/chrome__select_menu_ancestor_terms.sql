SELECT DISTINCT pt.taxonomy, pt.slug FROM post_terms pt
JOIN terms t ON t.taxonomy=pt.taxonomy COLLATE NOCASE AND t.slug=pt.slug
JOIN menu_items mi ON mi.item_type='taxonomy' AND mi.object_id=t.id
WHERE pt.post_id=?1
