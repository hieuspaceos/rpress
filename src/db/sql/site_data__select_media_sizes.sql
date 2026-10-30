SELECT path, COALESCE(sizes_json,'{}') FROM media WHERE wp_id=?1
