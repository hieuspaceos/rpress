INSERT INTO tours(post_id, duration_days, price_range, max_people, departure_dates_json, view_count, rating)
             VALUES(?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(post_id) DO UPDATE SET
               duration_days=excluded.duration_days, price_range=excluded.price_range,
               max_people=excluded.max_people, departure_dates_json=excluded.departure_dates_json,
               view_count=excluded.view_count, rating=excluded.rating
