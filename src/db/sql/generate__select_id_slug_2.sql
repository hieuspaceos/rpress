SELECT id, slug, lang, kind FROM posts
         WHERE status='publish' AND kind IN ('post','page','tour')
           AND slug NOT LIKE '%__trashed%'
