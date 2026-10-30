SELECT DISTINCT pt.taxonomy, pt.slug
             FROM post_terms pt JOIN posts p ON p.id=pt.post_id
             WHERE p.status='publish' AND p.kind IN ('tour','post','to_book')