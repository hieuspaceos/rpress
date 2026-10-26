UPDATE terms SET parent_id=(SELECT t2.parent_id FROM terms t2 WHERE t2.id=?1)
 WHERE parent_id=?1
