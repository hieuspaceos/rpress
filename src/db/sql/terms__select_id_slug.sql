SELECT t.id, t.slug, t.name, COALESCE(t.description,''),
       (SELECT COUNT(*) FROM post_terms pt WHERE pt.taxonomy=t.taxonomy AND pt.slug=t.slug),
       COALESCE(t.parent_id,0)
  FROM terms t WHERE t.id=?1
