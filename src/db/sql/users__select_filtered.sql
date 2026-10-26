SELECT id, email, role, COALESCE(display_name,''), COALESCE(first_name,''), COALESCE(last_name,''), COALESCE(website,''), created_at
FROM users
WHERE (?1='' OR role=?1)
  AND (?2='' OR email LIKE '%'||?2||'%' OR display_name LIKE '%'||?2||'%'
       OR first_name LIKE '%'||?2||'%' OR last_name LIKE '%'||?2||'%' OR website LIKE '%'||?2||'%')
ORDER BY lower(email)
LIMIT ?3 OFFSET ?4