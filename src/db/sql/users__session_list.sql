SELECT token, user_id, ua, ip, created_at, last_seen
FROM sessions WHERE user_id=?1 ORDER BY last_seen DESC