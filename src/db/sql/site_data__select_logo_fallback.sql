SELECT path, COALESCE(alt,'') FROM media WHERE LOWER(path) LIKE '%logo%' ORDER BY wp_id LIMIT 1
