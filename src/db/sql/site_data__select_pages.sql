SELECT id, slug, title, COALESCE(parent_id,0), COALESCE(menu_order,0)
FROM posts WHERE kind='page' AND lang=?1 AND status='publish'
ORDER BY menu_order, title
