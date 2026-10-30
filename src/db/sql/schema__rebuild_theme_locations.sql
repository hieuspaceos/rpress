ALTER TABLE theme_locations RENAME TO theme_locations_old;
CREATE TABLE theme_locations(
  location TEXT NOT NULL,
  lang TEXT NOT NULL DEFAULT '',
  menu_id INTEGER REFERENCES menus(id) ON DELETE SET NULL,
  PRIMARY KEY(location, lang)
);
INSERT INTO theme_locations(location, lang, menu_id)
  SELECT o.location, COALESCE((SELECT lang FROM menus WHERE id=o.menu_id),''), o.menu_id
  FROM theme_locations_old o;
DROP TABLE theme_locations_old;