UPDATE posts SET status='publish' WHERE status='future' AND published_at<=datetime('now')
