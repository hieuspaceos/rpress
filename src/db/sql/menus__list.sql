SELECT id, name, COALESCE(lang,''), COALESCE(location,''), COALESCE(auto_add,0) FROM menus ORDER BY id
