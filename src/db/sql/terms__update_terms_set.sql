UPDATE terms SET name=?1, slug=?2, description=?3, parent_id=NULLIF(?4,0) WHERE id=?5
