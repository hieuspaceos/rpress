CREATE TABLE wp_postmeta(post_id INT, meta_key TEXT, meta_value TEXT);
-- `_elementor` literal prefix: rows 1-2 match. `Xelementor_data` must
-- NOT match under instr semantics (it would under LIKE `_elementor%`).
INSERT INTO wp_postmeta VALUES
   (7,'_elementor_data','[]'),
   (7,'_elementor_css','x'),
   (7,'Xelementor_data','bad'),
   (999,'_elementor_data','ghost');
