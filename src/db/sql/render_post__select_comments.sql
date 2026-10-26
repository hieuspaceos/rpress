SELECT id, COALESCE(parent_id,0), COALESCE(author,''), COALESCE(author_url,''),
       COALESCE(body,''), COALESCE(created_at,''), COALESCE(rating,0),
       COALESCE(email,'')
FROM comments WHERE post_id=?1 AND status='approved' ORDER BY created_at ASC, id ASC
