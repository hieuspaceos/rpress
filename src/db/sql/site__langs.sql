-- check phase per-lang output: every lang dir must hold its own front page.
SELECT DISTINCT lang FROM posts WHERE status='publish' ORDER BY lang;
