SELECT name FROM terms WHERE taxonomy=?2 AND slug=?1
             UNION SELECT name FROM post_terms WHERE taxonomy=?2 AND slug=?1 LIMIT 1
