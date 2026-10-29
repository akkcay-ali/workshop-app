# Workshop – çalışma notları

Kullanıcı Türkçe çalışır; arayüz metinleri ve yorumlar Türkçe.

## Tek kaynak
- `web/workshop.html` hem claude.ai web artifact'ının (https://claude.ai/artifact/Tw3ADK5U82xEwBpQCESCDo)
  hem de masaüstü uygulamasının kaynağıdır. Önce bunu düzenle.
- Masaüstü için `python tools/build-frontend.py` → `src/index.html` üretir (Google Fonts yerine `src/fonts.css`,
  `src/vendor/fflate.js` eklenir). `src/index.html` elle düzenlenmez.
- Web artifact'ını güncellerken aynı dosya yayınlanır (artifact iskeleti zaten dosyada).

## Veri katmanı (`api` / `assets`)
- `S.mode`: `db` (artifact veritabanı), `demo` (önizleme), `file` (Tauri masaüstü).
- Dosya modu: her proje bir `.workshop` zip'i (`project.json` + `images/<assetId>.<ext>`), varsayılan
  `Belgeler\Workshop`. Kayıt 600 ms gecikmeli ve otomatik (`markDirty` → `saveProject`).
  Ayarlar ve projesiz kayıtlar localStorage'da (`settingsDoc`, `loose`).
- Rust komutları `src-tauri/src/lib.rs`: `library_dir`, `list_workshops`, `read_bytes`, `write_bytes`
  (ham gövde + `path` başlığı, geçici dosya üzerinden atomik yazım), `rename_file`, `trash_file`,
  `initial_files`. Çift tıklanan dosyalar `open-files` olayıyla gelir (single-instance).

## Bulut (ortak projeler)
- Supabase projesi `workshop` (ref `mbhpblbynyaotbblnxyr`). Tablolar: `projects`, `members`, `invites`, `profiles`, `items`
  (öğe = satır, `data` jsonb, silme = `deleted`), kova `assets` (`<pid>/<assetId>`). RLS: `private.is_member`.
  Davet: `invite_member(p, mail)` RPC; davet edilen e-posta `private.allowed_emails`'e eklenir.
  Kayıt yalnızca `private.allowed_emails` ya da `invites` içindeki e-postalara açık (`private.guard_signup` tetikleyicisi).
- İstemci: `CL` nesnesi, `wrapApi()` api/assets çağrılarını projeye göre yönlendirir (`isCloud(pid)`).
  `cloudOnly()` = GitHub Pages sürümü (window.claude yok, supabase-js var). claude.ai artifact'ında bulut kapalı.
- `src/vendor/supabase.js` (supabase-js UMD) build-frontend ile eklenir.
- Ekip (yalnızca ortak = üye/davet >1): görev `assignee`, `public.activity` (items tetikleyicisi `private.log_activity`
  ekle/düzenle/bitir/ata/sil kaydı), varlık kanalı `proj:<pid>` (özel, `realtime.messages` RLS), proje `brief` alanı,
  `profiles.name` görünen ad.

## Derleme
- Masaüstü derlemesi yalnızca `src-tauri/tauri.conf.json` değişince (sürüm artışı) çalışır.
- Web sürümü `pages.yml` ile GitHub Pages'e gider (arayüz değişince).
- Windows kurulumu GitHub Actions'ta (`.github/workflows/build.yml`) derlenir ve Release olarak yayınlanır.
  Yeni sürüm için `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` ve `package.json` içindeki
  `version` birlikte artırılır; aynı sürüm numarasıyla tekrar gönderim Release'i güncellemeyebilir.
- Kendi kendini güncelleme: `tauri-plugin-updater`, uç nokta `releases/latest/download/latest.json` (depo herkese açık olmalı).
  İmza için depo secret'ları gerekir: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
  Açık anahtar `tauri.conf.json > plugins.updater.pubkey` içinde; anahtar değişirse eski kurulumlar güncellenemez.
- macOS derlemesi elle tetiklenir (`mac.yml`). iOS derlemesi Mac'te yapılır (README).

## Kullanıcı tercihleri
- Doğrudan, sade çözümler; riskler baştan söylenir.
- Animasyonlar yaylı/"pop" hissinde; tema token'ları `:root` + `[data-theme=light|vivid]`.
