INSERT INTO forms(slug, title, lang, kind, markup, mail_to, mail_from, mail_subject, mail_body, messages, created_at, updated_at)
VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'), datetime('now'))
