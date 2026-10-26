SELECT featured_media_id, COALESCE(excerpt,''), COALESCE(seo_title,''), COALESCE(seo_desc,'')
             FROM posts WHERE id=?1 LIMIT 1