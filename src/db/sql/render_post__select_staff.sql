SELECT 1 FROM users WHERE lower(display_name)=?1 OR lower(email)=?1 LIMIT 1
