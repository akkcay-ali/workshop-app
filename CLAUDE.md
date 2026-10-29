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
- Merkez (`sHub`, ortak projelerde varsayılan sekme): kilometre taşı (`type:"milestone"`, görevde `milestone`),
  görev `status` (todo/doing/review/done, `done` ile eş) + Kanban, ayrıntı paneli `#drw` (yorum = `type:"comment"`,
  `target`, `mentions`), referans onayı (`card.review`), `public.notifications` (tetikleyici `private.notify_items`),
  sohbet `public.messages` (genel/özel, her pazartesi silinir: `private.prune_messages`).
- Sesli oda: WebRTC kişiden kişiye (`V` nesnesi), sinyal `proj:<pid>` kanalında broadcast `rtc`, odadakiler presence `v:true`.
  Bağlantı "perfect negotiation" (iki taraf teklif yapabilir). Ekran paylaşımı: tek paylaşan (presence `s`), kalite `SQ`
  (1080p30/720p30/720p60/Düşük, `vQ`), izleyici penceresi `#scr`. Ses/üst çubuk: kulaklık düğmesi `#voiceBtn` + `#voicePop`.
  TURN: edge function `turn-ice` (anahtarlar `CF_TURN_KEY_ID`, `CF_TURN_TOKEN`: function secret ya da Vault → `public.turn_secrets()`; yoksa yalnızca STUN). macOS mikrofon izni `src-tauri/Info.plist`.
- Görevler (ortak projede): sekmeler Merkez · Görevlerim (`tasks`: özel + bana atanan) · Ekip (`team`: Kanban, özel hariç) · panolar · Takvim.
  Özel görev = `data.private` + satırda `items.private_to` (RLS: yalnızca sahibi okur/yazar; akış ve bildirim tetikleyicileri atlar).
  "Ekibe aç" özel görevi ekip görevine çevirir ve bana atar. Merkez'de başlıkta hafta şeridi (ekip görevleri + kilometre taşları, gün tıklanınca `hubDay` listesi).
- Referanslar (ortak projede): tek "Panolar" sekmesi + solda dizin `#rdir` (`renderDir`): Ekip panoları / Benim panolarım.
  Özel pano `board.private`, üzerindeki kartlar da `private` (api.add otomatik), görselleri `<pid>/p/<uid>/<id>` (kartta `ap`; storage RLS yalnızca sahibine).
  Ekibe gönder: sağ tık ya da kartı dizindeki ekip panosunun adına sürükle (`sendToBoard`, dosya `storage.copy` ile ekip klasörüne); "Ekibe aç" panonun tamamını açar. Pano kilidi `board.locked`: adı değişmez, silinemez (`lockBoard`, sağ tık menüsü `boardMenu`). Pano sahibi `board.owner` (oluşturan): ortak projede yalnızca o siler, adını/kilidini/gizliliğini değiştirir (`canMng`; veritabanında `private.guard_board` tetikleyicisi). Diğerleri içeriği düzenler. Sahip "Gizliye al" (`boardToPrivate`) ile panoyu özele çevirir; ekipteki istemcilere broadcast `hide` gider. Dizin gizlenebilir: `setRdir` → `body.rdc`, LS `rdirC`.
- Presence: `presTrack` → `presFlush` (kanal başına yalnızca değişince, ≥1,2 sn arayla; Supabase istemci presence sınırı). Sesli odaya giriş/çıkış ayrıca broadcast `rtc` `join`/`here`/`bye` ile duyurulur.
- Her projede: görsele bağlı not (`card.note`), ses kartı (`kind:"audio"`, Drive linki → `uc?export=download`).

## Giriş ve pencere (masaüstü + web)
- Web (Pages) de aynı giriş ekranını gösterir; hesapsız modda yerel projeler tarayıcıda (öğeler localStorage `wloose`, görseller IndexedDB `ws-local` → `idbAssets`).
- İlk açılış: giriş ekranı `#welcome` (`maybeWelcome`, LS `welcome` = `acct`|`guest`); hesapsız devam = çevrimdışı, bir daha sorulmaz, giriş sağ üstten. Girişsizken ses/bildirim düğmeleri gizli.
- iPad/dokunmatik (`NOKBD`, `html.nokbd`): `kbd`, `.kbonly`, kısayol yardımı gizli; menü etiketlerindeki "· Ctrl+…" (`kbStrip`) ve bildirimlerdeki kısayol ipuçları atılır. Web ana ekrana eklenebilir (`manifest.webmanifest`, `apple-touch-icon.png`).
- Pencere × düğmesi görev çubuğuna küçültür; çıkış: × üzerinde sağ tık › Uygulamadan çık ya da Ctrl+Q (`onCloseRequested` bekleyen kayıtları bitirir).

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
