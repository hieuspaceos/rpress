UPDATE forms SET title=?1, markup=?2, mail_to=?3, mail_from=?4, mail_subject=?5, mail_body=?6,
  mail2_active=?7, mail2_to=?8, mail2_subject=?9, mail2_body=?10,
  messages=?11, additional=?12, updated_at=datetime('now')
WHERE id=?13
