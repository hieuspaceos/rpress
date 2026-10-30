CREATE TABLE wp_postmeta(post_id INT, meta_key TEXT, meta_value TEXT);
CREATE TABLE wp_options(option_name TEXT, option_value TEXT);
INSERT INTO wp_postmeta VALUES
   (42,'gdlr-core-page-builder','[{"a":1}]'),
   (42,'gdlr-core-skin','dark'),
   (42,'secret','x'),
   (43,'gdlr-core-skin','y');
INSERT INTO wp_options VALUES
   ('gdlr-core-templates','{"t":1}'),
   ('admin_email','a@b.c');
