CREATE TABLE wp_posts(ID INT, post_content TEXT, post_status TEXT, post_type TEXT);
-- ID 7 maps to rpress post 1; 8 → post 2 (gdlr body — untouched);
-- 9 → post 3 (no body_json); 999 resolves to nothing. The attachment
-- row exercises the pull filter (its small id MUST NOT collide).
INSERT INTO wp_posts VALUES
   (7,'<div>edited</div>','publish','tour'),
   (8,'<p>gdlr</p>','draft','page'),
   (9,'<p>fresh</p>','publish','tour'),
   (999,'ghost','publish','tour'),
   (1,'attachment-body','inherit','attachment');
