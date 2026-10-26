SELECT ap.id, u.id, u.email, u.role
FROM app_passwords ap JOIN users u ON u.id=ap.user_id
WHERE ap.hash=?1 AND u.email=?2
LIMIT 1