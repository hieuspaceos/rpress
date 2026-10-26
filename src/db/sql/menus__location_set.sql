INSERT INTO theme_locations(location, menu_id) VALUES(?1,?2)
ON CONFLICT(location) DO UPDATE SET menu_id=excluded.menu_id
