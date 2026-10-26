SELECT COUNT(*) FROM comments WHERE lower(email)=?1 AND status='approved'
