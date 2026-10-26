-- configure phase: record the dump's origin twice, point the site at LOCAL_URL.
--   ?1 = local url, ?2 = origin (asset_origin already imported, else old site_url)
INSERT OR REPLACE INTO settings (key, value) VALUES
  ('asset_origin', COALESCE(
    NULLIF((SELECT value FROM settings WHERE key='asset_origin'), ''),
    NULLIF(?2, ''),
    (SELECT value FROM settings WHERE key='site_url'))),
  ('legacy_origin', COALESCE(
    NULLIF((SELECT value FROM settings WHERE key='legacy_origin'), ''),
    NULLIF((SELECT value FROM settings WHERE key='asset_origin'), ''),
    NULLIF(?2, ''),
    (SELECT value FROM settings WHERE key='site_url'))),
  ('site_url', ?1);
