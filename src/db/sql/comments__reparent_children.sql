UPDATE comments
SET parent_id=(SELECT parent_id FROM comments WHERE id=?1)
WHERE parent_id=?1 AND id != ?1
