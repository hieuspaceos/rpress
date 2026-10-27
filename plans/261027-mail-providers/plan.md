---
title: "Mail subsystem — providers, newsletter, anti-spam"
description: "Selectable outbound mail backends (SMTP: Zoho/Gmail/Outlook/generic, HTTP API: SendGrid/Mailgun/SES), native newsletter (subscribers, double opt-in, compose-send), and anti-abuse (login rate-limit, form honeypot, outbound flood guard). Core feature, no adapter needed."
status: pending
priority: P2
effort: "1-2d"
tags: [mail, intake, forms, core]
created: 2026-10-27
---

# Mail providers

## Premise (user-confirmed 2026-10-27)

Form submissions (CF7-parity intake) must notify via real outbound mail.
Editor picks a provider + fills credentials in admin — done. This is a
**core** subsystem: every site with forms needs it; no plugin adapter
involved.

## Current state (scout before planning)

- `src/intake.rs` handles public form submit → `db/submissions.rs` inserts.
- `src/admin/` has settings screens; `SiteEnv` carries per-site env keys.
- No mail code exists today (verify — grep `mail|smtp|sendgrid`).

## Shape (to be validated by ak:plan pass)

```rust
trait Mailer: Send + Sync {
    fn send(&self, to: &str, subject: &str, body: &MailBody) -> Result<()>;
}
```

Providers:

| key | transport | creds |
|---|---|---|
| `smtp` | SMTP+TLS | host, port, user, pass — covers Zoho, Gmail (app password), Outlook, generic |
| `sendgrid` | HTTP API | api_key |
| `mailgun` | HTTP API | api_key, domain |
| `ses` | HTTP API | region, access_key, secret |
| `none` | default | log submission + eprintln only |

Config resolution order: `settings` table (admin UI) → `sites/<site>.env`
`MAIL_*` keys. Admin settings page: provider select + credential fields +
"Send test email" button.

## Wiring

- `intake.rs`: after submission insert → `mailer.send(notify_to)` where
  `notify_to` comes from form config or site admin email.
- **Never lose the submission**: mail failure logs + returns success to
  the visitor (submission already persisted); optionally record
  `mail_status` on the submission row for the admin inbox.
- Rate-limit/queue: out of scope for v1 — synchronous send with a short
  timeout; queue/retry is a later phase if needed.

## Newsletter (native — same subsystem)

- `subscribers` table (email, lang, confirmed_at, unsub token, source).
- Public subscribe endpoint reuses the intake path (`/form/newsletter`
  or dedicated route) → double opt-in mail via the configured provider.
- Admin: subscribers list + export CSV + compose-and-send (simple
  HTML/text issue) batched through the Mailer — v1 synchronous with a
  per-send cap; a send queue is a later phase if volume demands.
- Unsubscribe link with token → one-click confirm page.

## Anti-spam / anti-abuse (native — gates login + public forms)

- **Login rate-limit**: per-IP (and per-username) failed-attempt counter
  → exponential backoff / temporary lockout. Stored in settings or a
  small `rate_limits` table (no external dep). Admin sees lockouts.
- **Public form/intake**: honeypot hidden field + time-to-submit floor
  (reject <2s) + per-IP submit rate limit. Optional Turnstile/reCAPTCHA
  key field for sites wanting a real captcha (config-only, off default).
- **Outbound flood guard**: mail sends capped (max N submissions/hour/IP
  and max N mails/min) so a spam run can't drain provider quota or
  get the domain flagged.
- All checks server-side, cheap, no third-party calls on the public path.

## Non-goals

- Inbound mail parsing, bounce handling, marketing automation,
  list segmentation, drag-drop campaign builders.
- Templating engine — reuse the existing submission field dump format
  (or a minimal `{field}` substitution per form config).

## Success criteria

- [ ] `MAIL_PROVIDER=none` behaves exactly like today.
- [ ] SMTP provider sends through Zoho/Gmail test creds.
- [ ] SendGrid sends via API key.
- [ ] Mail failure does not drop or error the submission.
- [ ] Admin "send test" produces a visible pass/fail.
- [ ] Subscribe → confirm → send → unsubscribe flow works end-to-end.
- [ ] `cargo check` + `cargo test` clean.
- [ ] Login lockout after N failed attempts; form honeypot+rate-limit reject bots.

## To be phased by ak:plan

This file is the outcome contract. A full `/ak:plan` pass (scout +
phases + red-team) should produce `phase-01-*.md` files before cook —
deps: TLS/HTTP client choice (rustls vs native-tls; check what axum
already pulls), submission mail_status column, admin settings screen.
