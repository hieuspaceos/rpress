SELECT p.id FROM post_translations t
JOIN posts p ON p.id=CASE WHEN t.fr_post_id=?1 THEN t.es_post_id ELSE t.fr_post_id END
WHERE p.lang=?2 AND p.status='publish'
  AND (t.fr_post_id=?1 OR t.es_post_id=?1) LIMIT 1
