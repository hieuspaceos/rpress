SELECT COALESCE(meta_value,'0') FROM post_meta
WHERE post_id=?1 AND meta_key='wpdiscuz_post_rating_count' LIMIT 1
