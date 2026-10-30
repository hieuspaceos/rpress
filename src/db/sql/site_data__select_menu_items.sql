SELECT m.id, m.wp_term_id, m.name, m.lang, m.location, i.id, i.parent_id, i.ord,
COALESCE(NULLIF(i.label,''),
  CASE i.item_type
    WHEN 'post_type' THEN (SELECT title FROM posts WHERE id=i.object_id AND lang=m.lang LIMIT 1)
    WHEN 'post_type' THEN (SELECT title FROM posts WHERE id=i.object_id LIMIT 1)
    WHEN 'taxonomy' THEN (SELECT name FROM terms WHERE id=i.object_id LIMIT 1)
  END, i.label),
COALESCE(i.url,''), i.object_id, i.item_type, COALESCE(i.classes,''),
COALESCE(i.title_attr,''), COALESCE(i.mega,'') FROM menus m
JOIN menu_items i ON i.menu_id=m.id ORDER BY i.ord
