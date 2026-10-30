SELECT MAX(p.modified_at) FROM posts p JOIN post_terms pt
ON pt.post_id=p.id WHERE pt.taxonomy=?1 COLLATE NOCASE
AND pt.slug=?2 AND p.status='publish'
