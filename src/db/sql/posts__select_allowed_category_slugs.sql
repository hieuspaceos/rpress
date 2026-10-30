SELECT slug FROM terms WHERE taxonomy='category' AND (
  slug IN (SELECT slug FROM post_terms WHERE taxonomy='category' AND post_id=?2) OR
  slug IN (SELECT pt.slug FROM post_terms pt JOIN posts p ON p.id=pt.post_id
           WHERE pt.taxonomy='category' AND p.lang=?1) OR
  slug NOT IN (SELECT slug FROM post_terms WHERE taxonomy='category'))
