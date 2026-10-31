
CREATE TABLE IF NOT EXISTS posts(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  slug TEXT,
  lang TEXT,
  title TEXT,
  body_json TEXT,
  body_html TEXT,
  excerpt TEXT,
  status TEXT,
  parent_id INTEGER,
  menu_order INTEGER,
  published_at TEXT,
  modified_at TEXT,
  seo_title TEXT,
  seo_desc TEXT,
  seo_jsonld TEXT,
  featured_media_id INTEGER,
  kind TEXT,
  template TEXT
);

CREATE TABLE IF NOT EXISTS tours(
  post_id INTEGER PRIMARY KEY,
  duration_days INTEGER,
  departure_dates_json TEXT,
  price_range TEXT,
  max_people INTEGER,
  tour_time TEXT,
  gallery_json TEXT,
  rating REAL,
  view_count INTEGER
);


CREATE TABLE IF NOT EXISTS post_terms(
  post_id INTEGER,
  taxonomy TEXT,
  slug TEXT,
  name TEXT,
  PRIMARY KEY(post_id, taxonomy, slug)
);
CREATE INDEX IF NOT EXISTS idx_post_terms_tax ON post_terms(taxonomy, slug);
CREATE INDEX IF NOT EXISTS idx_post_terms_post ON post_terms(post_id, taxonomy);

CREATE TABLE IF NOT EXISTS terms(
  id INTEGER PRIMARY KEY,
  taxonomy TEXT NOT NULL,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT DEFAULT '',
  parent_id INTEGER,
  UNIQUE(taxonomy, slug)
);
CREATE TABLE IF NOT EXISTS post_translation_members(
  group_id TEXT NOT NULL,
  lang TEXT NOT NULL,
  post_id INT NOT NULL,
  PRIMARY KEY(group_id, lang),
  UNIQUE(post_id)
);
CREATE INDEX IF NOT EXISTS idx_post_translation_members_group ON post_translation_members(group_id);
CREATE INDEX IF NOT EXISTS idx_post_translation_members_lang ON post_translation_members(lang);

CREATE TABLE IF NOT EXISTS term_translations(
  fr_term_id INTEGER,
  es_term_id INTEGER,
  PRIMARY KEY(fr_term_id)
);

CREATE TABLE IF NOT EXISTS menus(
  id INTEGER PRIMARY KEY,
  wp_term_id INTEGER UNIQUE,
  name TEXT,
  location TEXT,
  lang TEXT,
  auto_add INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS menu_items(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  menu_id INTEGER,
  parent_id INTEGER,
  ord INTEGER,
  label TEXT,
  url TEXT,
  object_id INTEGER,
  item_type TEXT,
  title_attr TEXT NOT NULL DEFAULT '',
  target TEXT NOT NULL DEFAULT '',
  classes TEXT NOT NULL DEFAULT '',
  mega TEXT NOT NULL DEFAULT '',
  xfn TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS theme_locations(
  location TEXT NOT NULL,
  lang TEXT NOT NULL DEFAULT '',
  menu_id INTEGER REFERENCES menus(id) ON DELETE SET NULL,
  PRIMARY KEY(location, lang)
);

CREATE TABLE IF NOT EXISTS media(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  path TEXT,
  title TEXT,
  alt TEXT,
  caption TEXT,
  description TEXT,
  mime TEXT,
  w INTEGER,
  h INTEGER,
  sizes_json TEXT,
  owner TEXT,
  created TEXT
);

CREATE TABLE IF NOT EXISTS comments(
  id INTEGER PRIMARY KEY,
  wp_id INTEGER UNIQUE,
  post_id INTEGER,
  parent_id INTEGER,
  author TEXT,
  email TEXT,
  body TEXT,
  status TEXT,
  created_at TEXT,
  rating INTEGER
);

CREATE TABLE IF NOT EXISTS review_assignments(
  review_id INTEGER,
  post_id INTEGER,
  PRIMARY KEY(review_id, post_id)
);

CREATE TABLE IF NOT EXISTS redirects(
  old_slug TEXT,
  new_url TEXT,
  source TEXT
);
CREATE INDEX IF NOT EXISTS idx_redirects_old ON redirects(old_slug);

CREATE TABLE IF NOT EXISTS revisions(
  id INTEGER PRIMARY KEY,
  post_id INTEGER,
  saved_at TEXT,
  blob TEXT
);

CREATE TABLE IF NOT EXISTS users(
  id INTEGER PRIMARY KEY,
  email TEXT UNIQUE,
  pass_hash TEXT,
  role TEXT,
  created_at TEXT
);

CREATE TABLE IF NOT EXISTS submissions(
  id INTEGER PRIMARY KEY,
  kind TEXT,
  name TEXT,
  page TEXT,
  tour TEXT,
  data_json TEXT,
  ip TEXT,
  status TEXT,
  created_at TEXT
);
CREATE TABLE IF NOT EXISTS forms(
  id            INTEGER PRIMARY KEY,
  slug          TEXT NOT NULL UNIQUE,
  title         TEXT NOT NULL,
  lang          TEXT NOT NULL DEFAULT 'fr',
  kind          TEXT NOT NULL DEFAULT 'form',
  markup        TEXT NOT NULL,
  mail_to       TEXT NOT NULL DEFAULT '',
  mail_from     TEXT NOT NULL DEFAULT '',
  mail_subject  TEXT NOT NULL DEFAULT '',
  mail_body     TEXT NOT NULL DEFAULT '',
  mail2_active  INTEGER NOT NULL DEFAULT 0,
  mail2_to      TEXT NOT NULL DEFAULT '',
  mail2_subject TEXT NOT NULL DEFAULT '',
  mail2_body    TEXT NOT NULL DEFAULT '',
  messages      TEXT NOT NULL DEFAULT '{}',
  additional    TEXT NOT NULL DEFAULT '',
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings(
  key TEXT PRIMARY KEY,
  value TEXT
);

CREATE TABLE IF NOT EXISTS post_meta(
  post_id INTEGER,
  meta_key TEXT,
  meta_value TEXT,
  PRIMARY KEY(post_id, meta_key, meta_value)
);

CREATE TABLE IF NOT EXISTS app_passwords(
  id         INTEGER PRIMARY KEY,
  user_id    INTEGER NOT NULL,
  name       TEXT NOT NULL,
  hash       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  last_used  TEXT,
  last_ip    TEXT,
  UNIQUE(user_id, name)
);
CREATE INDEX IF NOT EXISTS idx_app_passwords_hash ON app_passwords(hash);

CREATE TABLE IF NOT EXISTS sessions(
  token      TEXT PRIMARY KEY,
  user_id    INTEGER NOT NULL,
  ua         TEXT NOT NULL,
  ip         TEXT NOT NULL,
  csrf       TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  last_seen  TEXT NOT NULL,
  expires_at INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);

-- WP-parity widget model (see plans/261026-0426-wp-parity-widgets): the
-- sidebars/widgets render pipeline keeps every frozen sidebar/footer
-- fragment in row-per-instance storage — a single settings row would
-- exceed the ~50KB audit threshold and make save/delete/order non-atomic.
CREATE TABLE IF NOT EXISTS sidebars(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  lang TEXT NOT NULL DEFAULT 'all'
);

CREATE TABLE IF NOT EXISTS widgets(
  id TEXT PRIMARY KEY,
  sidebar_id TEXT NOT NULL,
  type TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  settings_json TEXT NOT NULL DEFAULT '{}',
  ord INTEGER NOT NULL DEFAULT 0,
  lang TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_widgets_sidebar ON widgets(sidebar_id, ord);
CREATE INDEX IF NOT EXISTS idx_post_meta_key ON post_meta(meta_key);
-- post_meta got a (post_id, meta_key) uniqueness rule (audit M18): readers
-- do `LIMIT 1` per key, so dup rows were dead weight with an arbitrary
-- winner. Existing dbs are deduped in `dedupe_post_meta` BEFORE this DDL
-- runs — the keeper is MIN(rowid), the most likely old LIMIT-1 winner
-- (rowids are assigned in insert order). Behavior change: later insert
-- attempts for an existing (post_id, meta_key) are now ignored/error.
CREATE UNIQUE INDEX IF NOT EXISTS idx_post_meta_pair ON post_meta(post_id, meta_key);
-- Lookup indexes (audit M18): comment threads per post / parent chain,
-- revision lists per post, and the kind+status admin/front filters.
CREATE INDEX IF NOT EXISTS idx_comments_post ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent ON comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_revisions_post ON revisions(post_id);
CREATE INDEX IF NOT EXISTS idx_posts_kind_status_lang ON posts(kind, status, lang);
-- Slug equality is the hottest lookup: every permalink render + menu item.
CREATE INDEX IF NOT EXISTS idx_posts_slug_lang ON posts(slug, lang);
-- publish_due scans `status='future' AND published_at<=now` per admin request.
CREATE INDEX IF NOT EXISTS idx_posts_status_pub ON posts(status, published_at);
