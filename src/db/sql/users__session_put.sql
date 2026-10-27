INSERT INTO sessions(token, user_id, ua, ip, csrf, created_at, last_seen, expires_at)
VALUES(?1,?2,?3,?4,?5,datetime('now'),datetime('now'),unixepoch()+?6)
ON CONFLICT(token) DO UPDATE SET last_seen=datetime('now'), expires_at=excluded.expires_at