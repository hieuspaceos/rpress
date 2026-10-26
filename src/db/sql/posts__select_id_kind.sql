SELECT id, kind, title, slug, lang, COALESCE(excerpt,''), COALESCE(seo_title,''),
                    COALESCE(seo_desc,''), status, featured_media_id, COALESCE(body_json,''), COALESCE(body_html,'')
             FROM posts WHERE id=?1
