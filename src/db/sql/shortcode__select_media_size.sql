SELECT path, COALESCE(sizes_json,''), COALESCE(w,0), COALESCE(h,0) FROM media WHERE id=?1
