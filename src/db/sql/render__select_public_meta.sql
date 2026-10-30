SELECT DISTINCT meta_key, meta_value FROM post_meta WHERE post_id=?1 AND meta_key NOT LIKE '\_%' ESCAPE '\'
