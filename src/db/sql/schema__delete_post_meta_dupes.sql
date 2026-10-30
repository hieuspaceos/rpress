DELETE FROM post_meta WHERE rowid IN (
    SELECT rowid FROM (
        SELECT rowid, ROW_NUMBER() OVER (
            PARTITION BY post_id, meta_key
            ORDER BY meta_value IS NOT NULL, meta_value, rowid) rn
        FROM post_meta)
    WHERE rn > 1)