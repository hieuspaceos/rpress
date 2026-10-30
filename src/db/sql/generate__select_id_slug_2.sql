SELECT id, slug, lang, kind FROM posts
         WHERE status='publish' AND kind IN ('post','page','tour','to_book')
           AND slug NOT LIKE '%__trashed%'