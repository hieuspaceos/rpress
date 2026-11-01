SELECT COALESCE(wp_id, id), COALESCE(title,''), '', COALESCE(caption,''),
       'inherit', path, COALESCE(id,0), 0,
       COALESCE(created,''), COALESCE(created,''),
       COALESCE(path,''), COALESCE(mime,'')
FROM media
ORDER BY COALESCE(wp_id, id)
