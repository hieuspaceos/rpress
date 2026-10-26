SELECT
            COALESCE(SUM(CASE WHEN status!='trash' THEN 1 ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN status='publish' THEN 1 ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN status='draft' THEN 1 ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN status='trash' THEN 1 ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN status!='trash' AND lang='fr' THEN 1 ELSE 0 END),0),
            COALESCE(SUM(CASE WHEN status!='trash' AND lang='es' THEN 1 ELSE 0 END),0)
         FROM posts WHERE kind=?1
