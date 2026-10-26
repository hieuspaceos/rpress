SELECT count(*) FROM users
WHERE (?1='' OR role=?1)
  AND (?2='' OR email LIKE '%'||?2||'%' OR display_name LIKE '%'||?2||'%'
       OR first_name LIKE '%'||?2||'%' OR last_name LIKE '%'||?2||'%' OR website LIKE '%'||?2||'%')