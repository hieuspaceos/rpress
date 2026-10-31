-- Phase-2 → phase-3 upgrade: rewrite one pristine frozen custom_html row
-- to a live tour_list widget. Guarded on (?1) the exact phase-2
-- settings_json AND (?3) the widget id, so an admin-edited or re-imported
-- row is never touched. `title` is taken from the new payload's own
-- "title" field — the seed title is site data, not a code constant.
UPDATE widgets SET type='tour_list', title=json_extract(?2,'$.title'), settings_json=?2
WHERE id=?3 AND settings_json=?1
