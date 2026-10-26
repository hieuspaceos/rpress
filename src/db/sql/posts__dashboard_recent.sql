SELECT id, COALESCE(title,''), COALESCE(published_at, COALESCE(modified_at,''))
FROM posts
WHERE status='publish' AND kind='post'
ORDER BY COALESCE(published_at, modified_at) DESC, id DESC
LIMIT ?1
