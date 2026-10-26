SELECT i.id, i.parent_id, i.ord, COALESCE(i.label,''), COALESCE(i.url,''),
       i.object_id, COALESCE(i.item_type,''),
       p.id, COALESCE(p.title,''), COALESCE(p.slug,''), COALESCE(p.lang,''), COALESCE(p.kind,''),
       t.id, COALESCE(t.name,''), COALESCE(t.slug,''), COALESCE(t.taxonomy,''),
       COALESCE(i.title_attr,''), COALESCE(i.target,''), COALESCE(i.classes,''),
       COALESCE(i.xfn,''), COALESCE(i.description,'')
FROM menu_items i
LEFT JOIN posts p ON i.item_type='post_type' AND p.id = i.object_id
LEFT JOIN terms t ON i.item_type='taxonomy' AND t.id = i.object_id
WHERE i.menu_id=?1 ORDER BY i.ord, i.id
