SELECT id, COALESCE(title,''), slug FROM posts
WHERE kind='page' AND status='publish' ORDER BY title COLLATE NOCASE
