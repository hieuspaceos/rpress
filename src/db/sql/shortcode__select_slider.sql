SELECT p.title,
(SELECT meta_value FROM post_meta WHERE post_id=p.id AND meta_key='ml-slider_settings' LIMIT 1),
(SELECT meta_value FROM post_meta WHERE post_id=p.id AND meta_key='metaslider_slideshow_theme' LIMIT 1)
FROM posts p WHERE p.id=?1 AND p.kind='ml-slider'
