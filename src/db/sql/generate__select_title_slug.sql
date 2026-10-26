SELECT title, slug, published_at, excerpt FROM posts
             WHERE status='publish' AND kind='post' AND lang=?1
             ORDER BY published_at DESC LIMIT 20
