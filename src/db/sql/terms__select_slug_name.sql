SELECT slug, name, COUNT(*) FROM post_terms WHERE taxonomy=?1 GROUP BY slug, name ORDER BY 3 DESC LIMIT 60
