SELECT path, COALESCE(alt,''), COALESCE(title,''), COALESCE(sizes_json,''), COALESCE(caption,''), COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1
