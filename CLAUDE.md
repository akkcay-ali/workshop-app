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

## Derleme
- Windows kurulumu GitHub Actions'ta (`.github/workflows/build.yml`) derlenir ve Release olarak yayınlanır.
  Yeni sürüm için `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` ve `package.json` içindeki
  `version` birlikte artırılır; aynı sürüm numarasıyla tekrar gönderim Release'i güncellemeyebilir.
- macOS derlemesi elle tetiklenir (`mac.yml`). iOS derlemesi Mac'te yapılır (README).

## Kullanıcı tercihleri
- Doğrudan, sade çözümler; riskler baştan söylenir.
- Animasyonlar yaylı/"pop" hissinde; tema token'ları `:root` + `[data-theme=light|vivid]`.
