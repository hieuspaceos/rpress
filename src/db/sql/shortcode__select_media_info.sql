SELECT path, COALESCE(alt,''), COALESCE(caption,''), COALESCE(title,''),
COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1
