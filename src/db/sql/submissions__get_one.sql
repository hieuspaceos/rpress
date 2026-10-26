SELECT id, COALESCE(kind,''), COALESCE(name,''), COALESCE(page,''), COALESCE(tour,''),
       COALESCE(data_json,''), COALESCE(ip,''), COALESCE(form_slug,''),
       COALESCE(status,''), COALESCE(created_at,'')
FROM submissions WHERE id=?1
