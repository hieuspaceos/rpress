INSERT INTO posts(wp_id, slug, lang, title, body_json, body_html, excerpt, status,
             parent_id, menu_order, published_at, modified_at, seo_title, seo_desc, seo_jsonld,
             featured_media_id, kind, template)
         SELECT NULL, slug||'-copy', lang, title||' — Copy', body_json, body_html, excerpt,
             ?1, parent_id, menu_order, published_at, modified_at, seo_title, seo_desc, seo_jsonld,
             featured_media_id, kind, template FROM posts WHERE id=?2
