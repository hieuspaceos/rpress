SELECT COUNT(*) FROM submissions
WHERE {status_filter}
  AND (?1 = '' OR COALESCE(form_slug,'') = ?1)
  AND (?2 = '' OR name LIKE '%'||?2||'%' OR data_json LIKE '%'||?2||'%' OR page LIKE '%'||?2||'%')
  AND (?3 = '' OR COALESCE(kind,'') = ?3)
  AND (?4 = '' OR created_at >= ?4 || ' 00:00:00')
  AND (?5 = '' OR created_at <= ?5 || ' 23:59:59')