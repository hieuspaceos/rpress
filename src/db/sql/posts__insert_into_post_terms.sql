INSERT INTO post_terms(post_id,taxonomy,slug,name) SELECT ?1,taxonomy,slug,name FROM post_terms WHERE post_id=?2
