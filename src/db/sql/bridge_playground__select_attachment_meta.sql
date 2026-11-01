SELECT COALESCE(wp_id, id), path, COALESCE(w,0), COALESCE(h,0),
       COALESCE(sizes_json,''), COALESCE(alt,''), COALESCE(caption,'')
FROM media
ORDER BY COALESCE(wp_id, id)
