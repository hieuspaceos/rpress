INSERT INTO revisions(post_id, saved_at, blob)
    SELECT id, datetime('now'), json_object('title', title, 'excerpt', excerpt, 'body_json', body_json)
    FROM posts WHERE id=?1
