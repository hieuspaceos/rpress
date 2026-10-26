SELECT t.slug, t.taxonomy FROM terms t
             JOIN term_translations tt ON (
                 (tt.fr_term_id=?1 AND tt.es_term_id=t.id)
                 OR (tt.es_term_id=?1 AND tt.fr_term_id=t.id))
             LIMIT 1