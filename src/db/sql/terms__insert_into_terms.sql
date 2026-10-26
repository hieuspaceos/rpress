INSERT INTO terms(taxonomy,slug,name,description,parent_id) VALUES(?1,?2,?3,?4,NULLIF(?5,0))
             ON CONFLICT(taxonomy,slug) DO UPDATE SET name=excluded.name, description=excluded.description, parent_id=excluded.parent_id
