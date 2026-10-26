SELECT id, slug, lang, kind FROM posts WHERE status='future' AND published_at<=datetime('now')
