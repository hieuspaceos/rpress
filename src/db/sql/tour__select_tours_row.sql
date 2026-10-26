SELECT COALESCE(view_count,0), COALESCE(price_range,'')
             FROM tours WHERE post_id=?1 LIMIT 1