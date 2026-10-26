SELECT COALESCE(form_slug,''), COUNT(*) FROM submissions GROUP BY COALESCE(form_slug,'')
