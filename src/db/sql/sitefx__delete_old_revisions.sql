DELETE FROM revisions WHERE id IN (
  SELECT id FROM (
    SELECT id, ROW_NUMBER() OVER (PARTITION BY post_id ORDER BY saved_at DESC, id DESC) rn
    FROM revisions
  ) WHERE rn > 50);