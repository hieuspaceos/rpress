UPDATE sessions SET last_seen=datetime('now'), expires_at=MAX(expires_at, unixepoch()+43200)
WHERE token=?1 AND expires_at > unixepoch()