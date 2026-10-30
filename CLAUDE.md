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
  (1080p30/720p30/720p60/Düşük, `vQ`), izleyici penceresi `#scr` (varsayılan büyük, `scrQ` gerçek çözünürlük/fps/hız; paylaşanda `vsQ` + sınır nedeni). Ekran için codec: ekran kartı kodlayabiliyorsa o (`hwCodecs`, mediaCapabilities powerEfficient), yoksa VP9 > H264 > VP8 (`prefCodec`); göstergede ekran kartı/işlemci, karşı SDP'ye `x-google-start-bitrate=2500` (`fastStart`). Ses ve yayın merkezi: sağ kenar sekmesi `#voiceBtn.vtab` (odadakilerin avatarları) → sağda açılıp kapanan panel `#voicePop.vhub` (`vhOpen`, LS `vhub`); panel açıkken yayın panelde önizlenir (`#vhPrev`, `drawPrev`), büyük pencere `#scr` yalnızca "Büyük izle" ile (`S.scrPop`, `.scr.withhub`).
  Gürültü engelleme: Normal (tarayıcı) / Güçlü (RNNoise, LS `vNs`): `micStream` → `rnnWrap` (AudioWorklet 48 kHz; worklet `#rnnW`, wasm base64 `#rnnB`, @sapphi-red/web-noise-suppressor 0.3.5); Güçlüde tarayıcının engellemesi kapanır.
- Canlı imleçler: broadcast `cur` (`curSend` ~12/sn, yalnızca aynı sayfada biri varsa; `curTick` çizer, `#curs`). Panoda pano koordinatı, diğer sayfalarda sayfaya göre (x oran, y px + scroll). Sayfa anahtarı `subOf(pid)` (pano `b:<id>`).
  TURN: edge function `turn-ice` (anahtarlar `CF_TURN_KEY_ID`, `CF_TURN_TOKEN`: function secret ya da Vault → `public.turn_secrets()`; yoksa yalnızca STUN). macOS mikrofon izni `src-tauri/Info.plist`.
- Görevler (ortak projede): sekmeler Merkez · Görevlerim (`tasks`: özel + bana atanan) · Ekip (`team`: Kanban, özel hariç) · panolar · Takvim.
  Özel görev = `data.private` + satırda `items.private_to` (RLS: yalnızca sahibi okur/yazar; akış ve bildirim tetikleyicileri atlar).
  "Ekibe aç" özel görevi ekip görevine çevirir ve bana atar. Merkez'de başlıkta hafta şeridi (ekip görevleri + kilometre taşları, gün tıklanınca `hubDay` listesi).
- Referanslar (ortak projede): tek "Panolar" sekmesi + solda dizin `#rdir` (`renderDir`): Ekip panoları / Benim panolarım.
  Özel pano `board.private`, üzerindeki kartlar da `private` (api.add otomatik), görselleri `<pid>/p/<uid>/<id>` (kartta `ap`; storage RLS yalnızca sahibine).
  Ekibe gönder: sağ tık ya da kartı dizindeki ekip panosunun adına sürükle (`sendToBoard`, dosya `storage.copy` ile ekip klasörüne); "Ekibe aç" panonun tamamını açar. Pano kilidi `board.locked`: adı değişmez, silinemez (`lockBoard`, sağ tık menüsü `boardMenu`). Pano sahibi `board.owner` (oluşturan): ortak projede yalnızca o siler, adını/kilidini/gizliliğini değiştirir (`canMng`; veritabanında `private.guard_board` tetikleyicisi). Diğerleri içeriği düzenler. Sahip "Gizliye al" (`boardToPrivate`) ile panoyu özele çevirir; ekipteki istemcilere broadcast `hide` gider. Dizin gizlenebilir: `setRdir` → `body.rdc`, LS `rdirC`.
- Presence: `presTrack` → `presFlush` (kanal başına yalnızca değişince, ≥1,2 sn arayla; Supabase istemci presence sınırı). Sesli odaya giriş/çıkış ayrıca broadcast `rtc` `join`/`here`/`bye` ile duyurulur.
- Roller `members.role`: `owner` (yönetici) · `admin` (yetkili) · `editor` (üye). Proje ayarları `#shr` (`openShare`, sekmeler Üyeler/Kullanım, `setShTab`).
  Yönetici `set_member_role` ile yetki verir/alır (yalnızca owner). Davet ve davet iptali yönetici + yetkili (`private.is_admin`, `invite_member` `not_admin`);
  çıkarma: yönetici herkesi, yetkili yalnızca üyeleri (`members_delete`). İstemcide `CL.role`, `isAdmin(pid)`.
  Kullanım: `project_usage(p)` (yalnızca yetkili+; depolama, kişi başı, içerik, 30 gün etkinlik, plan doluluğu) → `drawUsage`; `purge_deleted(p)` 7 günden eski silinmişleri temizler.
  Sahipsiz panoları ortak projede proje sahibi yönetir.
- Yedek: bulut projesi menüsünde "Yedek indir (.workshop)" (`cloudBackup`, `zipItems` + `blobOf`, sürüm görselleri dahil `assetIdsOf`).
- İğneli yorum: yorumda `pin:{x,y}` (0–1); ayrıntı panelinde görsele tıkla (`pinAt`), panoda `.cpins`. Karşılaştır: iki görsel seç → `#cmp` (kaydır/üst üste/yan yana).
- Sürüm yığını: kartta `versions:[{assetId,ap,at,by}]`, `vAt`; `addVersion`/`verRestore`/`verCompare`; ekibe gönderirken yalnızca güncel sürüm gider.
- Video kartı `kind:"video"` (`vidInfo`: YouTube/Vimeo/Drive/mp4; tıklayınca iframe/video, `VPLAY`). Drive linkinde ses/video sorulur.
- Etiket `card.tags` (sağ tık), aramada `#etiket`; baskın renk `COLR` (yerel hesap, kaydedilmez) ile "mavi" gibi arama.
- Ekran alıntısı (masaüstü): Ctrl+Alt+W (global-shortcut, `snip` olayı) ya da araç çubuğu "Alıntı" → `snip_start` (Windows ms-screenclip / macOS screencapture) → `clip_seq`/`clip_image` (arboard+png) → açık ya da son panoya eklenir.
- Ekip bildirimleri `teamNotify` (LS `tnotify`, zil menüsünden): bahsetme/atama/onay/özel mesaj/adın geçtiği mesaj. Kalem basıncı: `draw` kartında `pr` dizisi, `inkOutline` dolu şekil.
- Kompozisyon katmanı: görsel kartta `guide` (thirds/golden/cross/grid, `GUIDES`/`GPATH`, `guideSync`, sağ tık `guideMenu`).
- Görselden görev: sağ tık › `openRTask` (`#rtk`, bölge sürükle) → görev `refs:[kart]` + `rbox:{id,x,y,w,h,ar}` (0–1); `cropHTML`/`rboxOf` Kanban kartında, görev satırında ve ayrıntı panelinde kırpılmış önizleme.
- Sesli not: araç çubuğu "Ses" (`recToggle`, MediaRecorder, en çok 5 dk) → ses kartı `assetId`+`mime`+`dur`+`author` (dosya projede; `auHost`, çalma `imgSrc`). Ses türleri `IMG_EXT`/`EXT_IMG`'de (zip).
- Pano şablonları: yeni pano = `openTpl` (`#tpk`) → `createBoard(pid,priv,tp)`; hazırlar `TPLS` (Ortam, 3D asset, Karakter, Ses tasarımı, Mood), `tplLayout` (brief notu + 3 sütun çerçeve).
  Özel şablon: pano sağ tık › "Şablon olarak kaydet" → öğe `type:"tpl"` (`project`, `name`, `cards`: çerçeve+not), projedeki herkes görür; `log_activity` `tpl`'yi atlar.
- Zamanlayıcı (`#timerPop`, sekmeler `setTTab`, LS `ttab`): Odak (Pomodoro, `TM`) · Kronometre (`SW`, LS `sw`; `swTask` görevde "süre", `swFinish` → `creditTask` → `task.focus` dk; görevde `est` tahmini dk, `timeBadge`) · Sayaçlar (`CT`, LS `ctimers`, `ctAdd` "40" dk ya da "15:30" saat; bitince `chime`+`notifyAlways`). Büyük sayı ve dakika kutuları fare tekerleğiyle değişir.
  Mini sayaç (masaüstü): `miniOpen` → `WebviewWindow("mini", index.html#mini)`, her zaman üstte; sayfa `html.mini` + `#miniRoot`, ana betik `__MINI` ile durur; durum olayı `mini-state`, komut `mini-cmd`; küçültünce/gizleyince otomatik (LS `t_mini`). Tray ipucu `trayTip`. İzinler `capabilities/desktop.json`, pencere listesi `default.json` (`mini`).
  Odakta: presence `f` (bitiş zamanı ya da 1 = kronometre) → `.hp.fx`; odaktayken `teamNotify` yalnızca acil (bahsetme, özel mesaj).
- Paylaş: yerel proje hemen taşınmaz (`shPre`); "Buluta taşı ve davet et" / "Yalnızca buluta taşı" (`shareMove`).
- Ses cihazları ayarlarda (`drawAud`, LS `vMic`/`vOut`); çan ve ses kartları da `vOut`'a çalar. Yazı tipi sabit: Geist + Unbounded (seçenek kaldırıldı).
- Her projede: görsele bağlı not (`card.note`), ses kartı (`kind:"audio"`, Drive linki → `uc?export=download`).
- Drive API anahtarı: `public.app_config` (`drive_key`, okuma herkes, yazma `set_app_config` yalnızca proje sahibi), istemcide LS `dkey`; varsa Drive ses/video `googleapis.com/drive/v3/files/<id>?alt=media&key=` ile akış (büyük dosya uyarısına takılmaz). Ses/video eklenince dosya adı çekilir (`mediaName`: Drive API `fields=name`, ya da linkteki dosya adı); adı boş eski kartlara bir kez yazılır (`fillName`). Anahtar uygulamada görünmez; Supabase panelinden `app_config` tablosuna (`key`=`drive_key`) girilir.

## Giriş ve pencere (masaüstü + web)
- Web (Pages) de aynı giriş ekranını gösterir; hesapsız modda yerel projeler tarayıcıda (öğeler localStorage `wloose`, görseller IndexedDB `ws-local` → `idbAssets`).
- İlk açılış: giriş ekranı `#welcome` (`maybeWelcome`, LS `welcome` = `acct`|`guest`); hesapsız devam = çevrimdışı, bir daha sorulmaz, giriş sağ üstten. Girişsizken ses/bildirim düğmeleri gizli.
- iPad/dokunmatik (`NOKBD`, `html.nokbd`): `kbd`, `.kbonly`, kısayol yardımı gizli; menü etiketlerindeki "· Ctrl+…" (`kbStrip`) ve parantez içi açıklamalar (`kbStrip`), bildirimlerdeki kısayol ipuçları atılır. Panoda iki parmakla dokun = geri al, üç parmak = yinele (`TT`); araç çubuğunda geri al/yinele düğmeleri (`.touchonly`). Uzun basma = sağ tık (dizin satırları ve proje sekmeleri dahil). Web ana ekrana eklenebilir (`manifest.webmanifest`, `apple-touch-icon.png`).
- Pencere × düğmesi simge alanına (saatin yanı, tray) gizler (`w.hide()`); tray: tık = aç, sağ tık menü Aç/Çıkış (`lib.rs` `TrayIconBuilder`, `tray-quit` olayı, 6 sn sonra zorla çıkış). Çıkış ayrıca × üzerinde sağ tık ya da Ctrl+Q (`onCloseRequested` bekleyen kayıtları bitirir).

## Derleme
- Masaüstü derlemesi yalnızca `src-tauri/tauri.conf.json` değişince (sürüm artışı) çalışır.
- Web sürümü `pages.yml` ile GitHub Pages'e gider (arayüz değişince).
- Windows kurulumu GitHub Actions'ta (`.github/workflows/build.yml`) derlenir ve Release olarak yayınlanır.
  Yeni sürüm için `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` ve `package.json` içindeki
  `version` birlikte artırılır; aynı sürüm numarasıyla tekrar gönderim Release'i güncellemeyebilir.
- Kendi kendini güncelleme: `tauri-plugin-updater`, uç nokta `releases/latest/download/latest.json` (depo herkese açık olmalı).
  İmza için depo secret'ları gerekir: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
  Açık anahtar `tauri.conf.json > plugins.updater.pubkey` içinde; anahtar değişirse eski kurulumlar güncellenemez.
- Release'e ek olarak `…_x64-offline-setup.exe` (WebView2 gömülü, `src-tauri/tauri.offline.conf.json`, güncelleme imzası yok) yüklenir.
- Açılış hatası sessiz kalmaz: `lib.rs` `fatal()` Windows mesaj kutusu; WebView2 yoksa (`tauri::webview_version`) indirme sayfasını önerir; panic kancası da aynı kutuyu gösterir.
- macOS derlemesi elle tetiklenir (`mac.yml`).
- iPad: `ios.yml` (elle) GitHub macOS makinesinde `tauri ios init/build` + TestFlight'a `altool` ile yükler. Kimlik `com.akkcay.workshop` (`tauri.ios.conf.json`), `Info.ios.plist` (mikrofon, şifreleme beyanı, Dosyalar). Secret'lar: `APPLE_DEVELOPMENT_TEAM`, `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_P8`. Masaüstüne özel izinler `capabilities/desktop.json`.

## Kullanıcı tercihleri
- Doğrudan, sade çözümler; riskler baştan söylenir.
- Animasyonlar yaylı/"pop" hissinde; tema token'ları `:root` + `[data-theme=light|vivid]`.
