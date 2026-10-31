SELECT COALESCE(body_html,''), status, COALESCE(body_json,'') FROM posts WHERE id=?1
