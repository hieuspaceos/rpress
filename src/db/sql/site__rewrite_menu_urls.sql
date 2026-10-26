-- Plain-text URL columns the renderer never rewrites get renamed in place.
-- ?1 = legacy origin, ?2 = local url
UPDATE menu_items SET url = REPLACE(url, ?1, ?2) WHERE url LIKE '%' || ?1 || '%';
