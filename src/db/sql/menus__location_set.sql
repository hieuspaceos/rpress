INSERT INTO theme_locations(location, lang, menu_id) VALUES(?1,?2,?3)
ON CONFLICT(location, lang) DO UPDATE SET menu_id=excluded.menu_id
