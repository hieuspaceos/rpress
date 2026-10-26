SELECT t.id, t.name,
       (SELECT COUNT(*) FROM post_terms pt WHERE pt.taxonomy=t.taxonomy AND pt.slug=t.slug) AS n
  FROM terms t
 WHERE t.taxonomy=?1 AND n > 0
 ORDER BY n DESC, t.name
 LIMIT 20
