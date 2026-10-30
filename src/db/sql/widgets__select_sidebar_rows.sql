SELECT id, type, title, settings_json FROM widgets
WHERE sidebar_id=?1 AND (lang=?2 OR lang='') ORDER BY ord