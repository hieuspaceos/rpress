UPDATE comments SET parent_id = (SELECT c2.parent_id FROM comments c2 WHERE c2.id = comments.parent_id)
 WHERE parent_id IN (SELECT id FROM comments WHERE status = ?1)
