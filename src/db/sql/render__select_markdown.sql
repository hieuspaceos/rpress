SELECT title, COALESCE(body_html,''), COALESCE(excerpt,''), COALESCE(published_at,''), kind, slug,
       COALESCE(body_json,'')
FROM posts WHERE slug=?1 AND (?2='' OR lang=?2) AND (?3='' OR kind=?3)
  AND status='publish' LIMIT 1
