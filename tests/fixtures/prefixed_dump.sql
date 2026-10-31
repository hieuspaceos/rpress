-- Minimal `wp_42426_*` prefixed mysqldump fixture for audit-remediation
-- tests (phase 4). Covers every WANTED table the canonicalize map used to
-- drop silently: termmeta, rank_math_redirections, glsr_ratings,
-- glsr_assigned_posts, plus users (author_login source), posts, postmeta,
-- terms, term_taxonomy, term_relationships, comments, options.
-- 1-2 rows each; wp_posts positional (23 cols, mysqldump order).

CREATE TABLE `wp_42426_options` (
  `option_id` bigint unsigned NOT NULL,
  `option_name` varchar(191) NOT NULL,
  `option_value` longtext,
  `autoload` varchar(20),
  PRIMARY KEY (`option_id`)
) ENGINE=InnoDB;
INSERT INTO `wp_42426_options` (`option_name`,`option_value`) VALUES
 ('siteurl','http://prefixed.test'),
 ('blogname','Prefixed');

INSERT INTO `wp_42426_users` VALUES
 (1,'admin','$P$Bhash','admin@x.t','http://x.t','2020-01-01 00:00:00','',0,'Admin');

INSERT INTO `wp_42426_terms` VALUES
 (1,'English','en',0),(2,'Espanol','es',0),(10,'Cat A','cat-a',0);
INSERT INTO `wp_42426_term_taxonomy` VALUES
 (1,1,'language','',0,0),(2,2,'language','',0,0),(10,10,'category','',0,0);
INSERT INTO `wp_42426_term_relationships` VALUES
 (100,1,0),(100,10,0);
INSERT INTO `wp_42426_termmeta` VALUES
 (1,2,'_pll_strings_translations','a:1:{i:0;a:2:{i:0;s:8:"Top text";i:1;s:10:"Texte haut";}}');

INSERT INTO `wp_42426_posts` VALUES
 (100,1,'2024-01-01 00:00:00','2024-01-01 00:00:00','<p>P</p>','Post A','','publish','closed','closed','','post-a','','','2024-01-02 00:00:00','2024-01-02 00:00:00','',0,'',0,'post','',0),
 (101,1,'2024-01-03 00:00:00','2024-01-03 00:00:00','','Rev','','inherit','closed','closed','','rev-100','','','2024-01-03 00:00:00','2024-01-03 00:00:00','',100,'',0,'revision','',0);
INSERT INTO `wp_42426_postmeta` VALUES
 (1,100,'_k','v');

INSERT INTO `wp_42426_comments` VALUES
 (1,100,'Alice','a@b.c','','','2024-01-10 00:00:00','2024-01-10 00:00:00','Hi',0,'1','ua','',0,0);
INSERT INTO `wp_42426_commentmeta` VALUES
 (1,1,'_ck','cv');

INSERT INTO `wp_42426_rank_math_redirections` VALUES
 (1,'a:1:{i:0;a:2:{s:7:"pattern";s:4:"/old";s:4:"type";s:5:"exact";}}','/new','301',0,'active','2024-01-01 00:00:00','2024-01-01 00:00:00','',0);
INSERT INTO `wp_42426_glsr_ratings` VALUES
 (1,100,0,5,'Great','','','1','2024-01-05 00:00:00');
INSERT INTO `wp_42426_glsr_assigned_posts` VALUES
 (1,100,'post',100);
