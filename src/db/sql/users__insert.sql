INSERT INTO users(email, pass_hash, role, created_at, display_name, first_name, last_name, website)
VALUES(?1,?2,?3,datetime('now'),?4,?5,?6,?7)