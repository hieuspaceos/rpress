WITH RECURSIVE d(id) AS (
  SELECT ?1
  UNION ALL
  SELECT t.id FROM terms t JOIN d ON t.parent_id=d.id
)
SELECT id FROM d
