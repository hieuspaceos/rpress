-- deploy staging: point the copied db at the public domain before generate.
UPDATE settings SET value=?1 WHERE key='site_url';
