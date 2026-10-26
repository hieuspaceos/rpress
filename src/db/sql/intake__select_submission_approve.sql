SELECT kind, COALESCE(name,''), COALESCE(page,''), COALESCE(data_json,'{}'), COALESCE(ip,'') FROM submissions WHERE id=?1
