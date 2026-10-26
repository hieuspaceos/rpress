SELECT id, user_id, name, created_at, COALESCE(last_used,''), COALESCE(last_ip,'')
FROM app_passwords WHERE user_id=?1 ORDER BY created_at DESC, id DESC