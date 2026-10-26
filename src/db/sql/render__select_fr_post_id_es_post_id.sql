SELECT fr_post_id, es_post_id FROM post_translations
             WHERE fr_post_id=?1 OR es_post_id=?1 LIMIT 1
