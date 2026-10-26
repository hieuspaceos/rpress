SELECT c.id, c.post_id, c.parent_id, COALESCE(c.author,''), COALESCE(c.email,''),
       COALESCE(c.author_url,''), COALESCE(c.author_ip,''), COALESCE(c.body,''),
       COALESCE(c.status,''), COALESCE(c.created_at,''),
       COALESCE(p.title,''), COALESCE(p.slug,''), COALESCE(p.kind,''), COALESCE(p.lang,''),
       (SELECT COUNT(*) FROM comments x WHERE x.post_id=c.post_id AND x.status='approved'),
       (SELECT COUNT(*) FROM comments x WHERE x.post_id=c.post_id AND x.status='pending')
FROM comments c LEFT JOIN posts p ON p.id=c.post_id
WHERE 1=1
