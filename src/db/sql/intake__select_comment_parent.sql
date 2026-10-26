SELECT parent_id FROM comments WHERE id=?1 AND post_id=?2 AND status='approved' LIMIT 1
