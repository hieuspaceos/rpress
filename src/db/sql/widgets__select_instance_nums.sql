SELECT CAST(substr(id, length(?1) + 2) AS INTEGER) FROM widgets
WHERE id LIKE (?1 || '-%')
  AND substr(id, length(?1) + 2) GLOB '[0-9]*'
  AND CAST(substr(id, length(?1) + 2) AS TEXT) = substr(id, length(?1) + 2)