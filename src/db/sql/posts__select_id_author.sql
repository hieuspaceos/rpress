SELECT id, author, substr(body,1,80), status FROM comments ORDER BY id DESC LIMIT ?1
