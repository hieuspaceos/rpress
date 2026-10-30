CREATE TABLE posts(id INTEGER PRIMARY KEY, wp_id INTEGER UNIQUE, slug TEXT,
    lang TEXT, kind TEXT, title TEXT, status TEXT);
CREATE TABLE terms(id INTEGER PRIMARY KEY, taxonomy TEXT, slug TEXT, name TEXT);
CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE post_terms(post_id INTEGER, taxonomy TEXT, slug TEXT);
INSERT INTO settings VALUES('languages','fr,es'),('primary_lang','fr'),
   ('page_on_front','0'),('site_url','https://origin.test');
INSERT INTO posts VALUES(1,101,'hello','fr','post','Hello','publish');
INSERT INTO posts VALUES(2,102,'draft','fr','post','Draft','draft');
INSERT INTO terms VALUES(7,'region','voyage-au-vietnam','Voyage au Vietnam');
INSERT INTO terms VALUES(8,'post_tag','news','News');
INSERT INTO post_terms VALUES(1,'region','voyage-au-vietnam');
