SELECT s.token, s.user_id, u.email, u.role, COALESCE(s.csrf,'')
FROM sessions s JOIN users u ON u.id=s.user_id
WHERE s.token=?1 AND s.expires_at > unixepoch()
LIMIT 1