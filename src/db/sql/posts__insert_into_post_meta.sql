INSERT INTO post_meta(post_id,meta_key,meta_value) SELECT ?1,meta_key,meta_value FROM post_meta WHERE post_id=?2
