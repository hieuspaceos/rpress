SELECT slug, kind FROM posts WHERE id=?1 AND status='publish' AND slug NOT LIKE '%__trashed%'
