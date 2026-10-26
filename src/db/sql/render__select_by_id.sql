SELECT lang, title, body_json, seo_title, seo_desc, kind
             FROM posts WHERE id=?1 AND status!='trash' LIMIT 1
