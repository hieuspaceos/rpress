SELECT DISTINCT meta_key, meta_value FROM post_meta WHERE post_id=?1 AND meta_key LIKE 'rank\_math\_%' ESCAPE '\'
