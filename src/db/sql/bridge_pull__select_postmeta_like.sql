-- Literal prefix match via instr (kept `_like` filename — churn over accuracy).
SELECT post_id, meta_key, meta_value FROM wp_postmeta WHERE instr(meta_key, ?1) = 1
