SELECT id, email, role, COALESCE(display_name,''), COALESCE(first_name,''), COALESCE(last_name,''), COALESCE(website,''), created_at
FROM users WHERE id=?1