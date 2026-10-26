SELECT id, saved_at, length(blob) FROM revisions WHERE post_id=?1 ORDER BY saved_at DESC, id DESC
