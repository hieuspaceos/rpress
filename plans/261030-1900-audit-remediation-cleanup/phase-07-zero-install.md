# Phase 7 — Zero-install: 1 lệnh → mọi thứ trong browser

Mục tiêu: người dùng **không biết VPS/Linux/DNS** vẫn cài được rpress.
Ràng buộc thật: không thể tránh hoàn toàn 1 lần mở terminal (ai đó phải mua
VPS và paste 1 lệnh) — trừ khi có marketplace image. Nên thiết kế là:

```
mua VPS → paste 1 dòng → đóng terminal vĩnh viễn → mọi thứ còn lại trong browser
```

## Lộ trình 3 mức

| Mức | Ai | Cách |
|---|---|---|
| A (đã có) | biết terminal | `install.sh` — binary/systemd/nginx block in sẵn |
| B (phase này) | biết mua VPS, không biết Linux | 1 lệnh → **web wizard** hoàn tất phần còn lại |
| C (sau) | không biết cả VPS | DO/Contabo marketplace image → click → wizard B |

## 7.1 Wizard first-boot — `rpress setup`

Command mới `rpress setup [--bind 0.0.0.0:80]`:

- Server axum tối giản (~300 LOC), KHÔNG cần `sites/*.env` — phục vụ đúng 1
  trang `/setup` + POST handlers. Tắt hẳn khi env đã tồn tại.
- `install.sh` gọi `rpress setup` khi `sites/` rỗng (thay vì `site init`
  interactive): unit systemd `rpress-setup.service` oneshot→ hoặc đơn giản
  hơn `rpress site serve` được mở rộng — chọn: **`site serve` khi chưa có
  env sẽ tự serve wizard**, khỏi thêm service.

### Trang wizard (thứ tự màn hình)

1. **Chọn đường**: "Site mới" | "Đưa site WordPress cũ qua".
2. **Site mới**: slug + domain + email admin + password → tạo `sites/<slug>.env`
   (LOCAL_URL/SITE_URL/HOST/BIND), `work/<slug>/rpress.db` qua `schema::migrate`,
   `auth::add_user`, `generate` homepage trống → redirect `/admin`.
3. **WP cũ**: upload `.sql`/`.sql.gz`/`.tar.gz` backup (multipart, streaming
   vào `work/<slug>/dump` — cùng code path `site_init` source=archive) →
   chạy `import::run` + `generate` trên thread nền, màn hình progress qua
   SSE/polling (`/setup/status` trả JSON phase). Không block worker.
4. **Domain + HTTPS** (màn riêng, skip được):
   - Option 1 — **Cloudflare Tunnel** (mặc định): user dán tunnel token từ
     dashboard CF → rpress spawn `cloudflared` (tải binary lần đầu) với
     token → domain+HTTPS xong, không đụng DNS/nginx/certbot.
   - Option 2 — **Manual**: hiện block nginx + lệnh `certbot --nginx -d X`
     để user copy (nhóm semi-chuyên).
   - Không nhúng ACME client: thêm ~1 crate lớn + edge cases (rate limit,
     port-80 occupied) — Caddy/cloudflared che cùng chỗ đó với ít rủi ro.
5. **Xong**: panel "admin tại /admin — bookmark lại".

### Quyết định thiết kế

- Wizard viết env bằng `SiteEnv::save()` (đã emit SITE_URL sau 1.1).
- Upload cap 2GB (`Content-Length` gate) — dump thật ~300MB.
- Progress import chạy `tokio::task::spawn_blocking`; state trong
  `Arc<Mutex<SetupState>>`, `/setup/status` JSON.
- CSRF: wizard là pre-auth → token trong session cookie tự mint
  (không dùng admin session; setup không share auth code).
- Khi init xong: unit file được `daemon-reload` + đổi `ExecStart` sang
  `site serve <slug>` (install.sh viết unit cuối cùng từ đầu, wizard chỉ
  tạo env; `Restart=always` tự đưa site lên).

## 7.2 Cloudflare Tunnel helper — `src/admin/setup.rs` + `cloudflared` fetch

- `POST /setup/tunnel {token}`: ghi `TUNNEL_TOKEN` vào env; systemd unit
  thêm `ExecStartPost`? Không — đơn giản: wizard spawn `cloudflared tunnel
  run --token T` như supervised child (cùng cơ chế bridge runtime), unit
  `rpress-cloudflared-<slug>.service` riêng viết bởi install script khi
  token tồn tại.
- `cloudflared` tải từ GitHub releases lần đầu (cache `~/.rpress/bin/`).

## 7.3 install.sh v2

- Hỏi duy nhất: "site slug?" (default mysite) — domain/dump chuyển hết vào wizard.
- Viết unit `rpress-<slug>` trỏ `site serve`; `site serve` phát hiện env
  chưa đầy → chạy wizard mode.
- In ra: "mở http://<ip-vps>:<port> trong browser để hoàn tất" — copy-paste được.

## 7.4 Sau phase này

- Marketplace image: snapshot DO droplet đã chạy install.sh + unit bật →
  publish image; user bấm Create → wizard hiện ngay. Không code mới.

## Acceptance

- Trên VPS sạch: `curl install.sh | bash` → mở `http://ip` → chọn "WP cũ"
  → upload dump → site chạy; không lệnh nào ngoài 1 lệnh curl.
- "Site mới" path: admin đăng nhập được ngay, homepage render.
- Tunnel path: domain https:// resolve về rpress sau khi dán token.
