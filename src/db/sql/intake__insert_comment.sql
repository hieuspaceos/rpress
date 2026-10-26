INSERT INTO comments(id, wp_id, post_id, parent_id, author, email, body, author_url, author_ip, status, created_at)
         VALUES(?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
