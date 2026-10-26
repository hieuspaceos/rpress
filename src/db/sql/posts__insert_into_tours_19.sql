INSERT INTO tours(post_id, duration_days, price_range, max_people, departure_dates_json)
             VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(post_id) DO UPDATE SET duration_days=excluded.duration_days,
               price_range=excluded.price_range, max_people=excluded.max_people,
               departure_dates_json=excluded.departure_dates_json
