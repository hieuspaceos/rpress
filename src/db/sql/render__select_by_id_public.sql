SELECT lang, title, body_json, seo_title, seo_desc, kind, status
             FROM posts WHERE id=?1 AND status='publish' LIMIT 1