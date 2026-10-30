SELECT path FROM media WHERE LOWER(path) LIKE '%' || ?1 || '%'
