SELECT COALESCE(status,''), COUNT(*) FROM submissions GROUP BY COALESCE(status,'')
