SELECT id FROM posts WHERE lang=?1 AND kind='page' AND slug LIKE '%__trashed%' AND status='publish' ORDER BY id LIMIT 1
