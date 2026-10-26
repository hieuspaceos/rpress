SELECT id, lang, title, body_json, seo_title, seo_desc, kind, status
             FROM posts WHERE slug=?1 AND (?2='' OR lang=?2) AND (?3='' OR kind=?3)
             AND status IN ('publish','draft') LIMIT 1