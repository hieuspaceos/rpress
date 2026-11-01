SELECT ID, post_content, post_status FROM wp_posts
WHERE post_type NOT IN ('attachment', 'revision', 'auto-draft')
