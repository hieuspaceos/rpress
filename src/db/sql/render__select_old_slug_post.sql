SELECT pm.post_id FROM post_meta pm
             JOIN redirects r ON r.old_slug=pm.meta_value AND r.source='old-slug'
             WHERE pm.meta_key='_wp_old_slug' AND r.new_url=?1
               AND (SELECT COUNT(DISTINCT m.post_id) FROM post_meta m
                    WHERE m.meta_key='_wp_old_slug' AND m.meta_value=pm.meta_value
                    AND m.rowid < (SELECT MIN(rowid) FROM post_meta
                        WHERE post_id=pm.post_id AND meta_key='_wp_old_slug'))
                 = (SELECT COUNT(*) FROM redirects r2
                    WHERE r2.old_slug=r.old_slug AND r2.source='old-slug'
                    AND r2.rowid < r.rowid)
             LIMIT 1
