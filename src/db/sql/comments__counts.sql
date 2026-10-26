SELECT
  SUM(CASE WHEN status IN ('approved','pending') THEN 1 ELSE 0 END),
  SUM(CASE WHEN status='pending'  THEN 1 ELSE 0 END),
  SUM(CASE WHEN status='approved' THEN 1 ELSE 0 END),
  SUM(CASE WHEN status='spam'     THEN 1 ELSE 0 END),
  SUM(CASE WHEN status='trash'    THEN 1 ELSE 0 END),
  -- "Mine" view count: caller's email in ?1; '' binds nothing.
  SUM(CASE WHEN ?1 != '' AND email=?1 AND status IN ('approved','pending')
           THEN 1 ELSE 0 END)
FROM comments
