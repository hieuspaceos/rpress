SELECT COALESCE(NULLIF(display_name,''), email) FROM users WHERE id=?1
