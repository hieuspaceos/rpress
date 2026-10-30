SELECT COUNT(*) FROM posts p WHERE NOT EXISTS (
  SELECT 1 FROM settings s WHERE s.key='languages'
    AND (','||s.value||',') LIKE ('%,'||p.lang||',%'))