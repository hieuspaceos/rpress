INSERT INTO sessions(token, user_id, ua, ip, created_at, last_seen, expires_at)
VALUES(?1,?2,?3,?4,datetime('now'),datetime('now'),unixepoch()+43200)
ON CONFLICT(token) DO UPDATE SET last_seen=datetime('now'), expires_at=unixepoch()+43200