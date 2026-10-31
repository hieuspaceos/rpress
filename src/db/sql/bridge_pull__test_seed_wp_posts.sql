CREATE TABLE wp_posts(ID INT, post_content TEXT, post_status TEXT);
-- ID 7 maps to rpress post 1; 8 → post 2 (gdlr body — untouched);
-- 9 → post 3 (no body_json); 999 resolves to nothing.
INSERT INTO wp_posts VALUES
   (7,'<div>edited</div>','publish'),
   (8,'<p>gdlr</p>','draft'),
   (9,'<p>fresh</p>','publish'),
   (999,'ghost','publish');
