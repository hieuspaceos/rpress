SELECT pt.slug, COUNT(*) FROM post_terms pt JOIN posts p ON p.id=pt.post_id WHERE pt.taxonomy=?1 AND p.status='publish' AND p.lang=?2 GROUP BY pt.slug
