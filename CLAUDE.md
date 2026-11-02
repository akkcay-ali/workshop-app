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
  sohbet `public.messages` (genel/özel, 7 gün sonra silinir: `private.prune_messages`, istemci `weekStartISO` = şimdi−7 gün).
- Sesli oda: WebRTC kişiden kişiye (`V` nesnesi), sinyal `proj:<pid>` kanalında broadcast `rtc`, odadakiler presence `v:true`.
  Bağlantı "perfect negotiation" (iki taraf teklif yapabilir). Ekran paylaşımı: tek paylaşan (presence `s`), kalite `SQ`
  (çözünürlük 2K(1440p, yalnızca 30 fps, 8 Mbps)/1080p/720p/480p × 30/60 fps, `vqKey`/`vqParts`; 480p30 anahtarı `low`; LS `vQ`), izleyici penceresi `#scr` (varsayılan büyük, `scrQ` gerçek çözünürlük/fps/hız; paylaşanda `vsQ` + sınır nedeni). Ekran için codec: ekran kartı kodlayabiliyorsa o (`hwCodecs`, mediaCapabilities powerEfficient), yoksa VP9 > H264 > VP8 (`prefCodec`); göstergede ekran kartı/işlemci, karşı SDP'ye `x-google-start-bitrate=2500` (`fastStart`). Ses ve yayın merkezi: sağ kenar sekmesi `#voiceBtn.vtab` (odadakilerin avatarları) → sağda açılıp kapanan panel `#voicePop.vhub` (`vhOpen`, LS `vhub`); panel açıkken yayın panelde önizlenir (`#vhPrev`, `drawPrev`), büyük pencere `#scr` yalnızca "Büyük izle" ile (`S.scrPop`, `.scr.withhub`).
  Odalar: her ortak projede hazır "Genel" ses + "genel" metin odası (id `main`); Genel odalar da yeniden adlandırılır/silinir (yetkili+; öğe `type:"room",main:true,kind,name,hidden` varsayılanı ezer, `mainOv`/`setMainRoom`; metin odası kalmazsa `vtSel` null, sohbet kapalı); ekip ekler: öğe `type:"room"` (`kind` voice|text, `name`, `by`; `roomsOf`, `addRoom`, sağ tık `roomMenu` yeniden adlandır/sil — açan ya da yetkili; `log_activity` `room`'u atlar).
  Ses odası: `V.room`, presence `v` = oda kimliği (eski istemcilerde true → main), rtc mesajlarında `room` (farklı odanınkiler yok sayılır), `voiceIn(pid,room)`, `roomIn()`, `voiceJoin(pid,room)`.
  Metin odası: `public.messages.room` (null = genel; Merkez'deki sohbet bloğu kaldırıldı, sohbet yalnızca ses merkezinde), seçili oda LS `vtxt` (`vtSel`), okunmamış LS `vts:<pid>:<oda>`; panel altında `#vChat` (`drawVChat`, aynı içerikte yeniden çizmez; `L.dataset.k`); ekler: ataç `#vChAtt` / sürükle / yapıştır → `chatUpload` → yalnızca R2 `c/<pid>/<id>.<ext>` (Supabase'e düşmez, XHR ilerleme `#vChUp`), görseller her zaman `chatImg` (WebP ≤2048), diğerleri ≤200 MB (`CH_MAX`); mesajda `att` {path,name,size,type,w,h}, metin = açıklama ya da dosya adı; `attHTML`/`attFill` imzalı GET (`CATT` 50 dk). Silme: mesajlar 7 gün sonra, dosyalar R2 yaşam döngüsü kuralı (`c/`, 7 gün); yükleyene uyarı gösterilir. Odadakiler yalnızca simge (`.vrow.vic`, ad/konum ipucunda); sohbette gönderen simge (`.mav`, 5 dk içinde ardışık mesajlar gruplanır); boyut: sohbet üst kenarı `#vchGrip` (LS `vchH`), panel sol kenarı `#vhGrip` (LS `vhW`, kökte `--vhw`), çift tık sıfırlar; sohbet en fazla son metin odasının altına kadar büyür; sol üstte `#vChPop` → `chatDock` sohbeti merkezin soluna ikinci çekmeceye `#vchDock` taşır (LS `vchOut`, ≤760px kapalı); çekmece genişliği `#vdGrip` (LS `vdW`, `--vdw`); `#vChWin` → `chatWin` sohbeti açılır pencereye taşır (`CHW`, `$` orada da arar, stiller kopyalanır, kapanınca `chatWinBack`); `convOf` oda mesajlarını Merkez sohbetinden ayırır.
  Ses merkezi ekleri: kim nerede (`locOf`: presence `w` sekme/pano + `f` odak) · takip et (satıra tık `followToggle`, rtc `fol` on/off → liderin `V.fans`; lider `viewSend` rtc `vw` {pid,sub,x,y,z,w,h}, takipçi `onView` merkezi eşler; panoya dokununca bırakır; `#folBar`) ·
  kişi başı ses (sağ tık `volPop`, LS `vVol`/`vPm`, `applyVol`) · bas-konuş (LS `vPtt`, `pttKey`) ve kısayollar Ctrl+Alt+M/D/V (masaüstü global-shortcut → `vkey`; web'de pencere odaktayken keydown); bas-konuş anahtarı Ayarlar'da `#pttSw` (hub'da kısayol satırı yok) ·
  mikrofon seviye halkası (`--lvl`, `voiceLoop`).
  Mikrofon/kulaklık sesi `vSnd`/`vSndSend` (rtc `snd`, odadakiler de duyar; bas-konuşta yok, `snd` ayarına uyar). Kulaklık kapat `voiceDeaf` (V.deaf: herkesin sesi kısılır, mikrofon da kapanır, açınca önceki hâline döner; presence `d`). Ses ayarları menüsü yalnızca cihazlar. Alt sıra simgeler: mikrofon · kulaklık · gürültü · ekran (`scrBtn` → `scrPop` kalite + Başlat/Durdur, `S.scrOpen`) · ekip odağı (`tfBtn`, açıkken `tfLine`) · ses ayarları (simge) · Ayrıl; paylaşım sürerken `shareLine`.
  Gürültü engelleme: ses panelinde küçük düğme `nsBtn` → seçici `nsPop` (S.nsOpen); Kapalı / Normal (tarayıcı) / RNNoise · Normal (`rnn`: %30 ham ses 10 ms gecikmeyle karışır, `rnnWrap(raw,dry)`) / RNNoise · Güçlü; LS `vNsM` (off|normal|rnn|strong, `nsMode`; eski `vNs` okunur): `micStream` → `rnnWrap` (AudioWorklet 48 kHz; worklet `#rnnW`, wasm base64 `#rnnB`, @sapphi-red/web-noise-suppressor 0.3.5); Güçlüde tarayıcının engellemesi kapanır.
- Canlı imleçler: broadcast `cur` (`curSend` ~12/sn, yalnızca aynı sayfada biri varsa; `curTick` çizer, `#curs`). Panoda pano koordinatı, diğer sayfalarda sayfaya göre (x oran, y px + scroll). Sayfa anahtarı `subOf(pid)` (pano `b:<id>`); sekme değişince `renderSub` → `presTrack` (yoksa iki kişi aynı sayfada olsa da imleç gitmiyordu).
  TURN: edge function `turn-ice` (anahtarlar `CF_TURN_KEY_ID`, `CF_TURN_TOKEN`: function secret ya da Vault → `public.turn_secrets()`; yoksa yalnızca STUN). macOS mikrofon izni `src-tauri/Info.plist`.
- Görevler (ortak projede): sekmeler Merkez · Görevlerim (`tasks`: özel + bana atanan) · Ekip (`team`: Kanban, özel hariç) · panolar · Takvim.
  Özel görev = `data.private` + satırda `items.private_to` (RLS: yalnızca sahibi okur/yazar; akış ve bildirim tetikleyicileri atlar).
  "Ekibe aç" özel görevi ekip görevine çevirir ve bana atar. Merkez'de "Görevlerim" = bana atanan ekip görevleri + özel görevlerim (kilit simgesi). Merkez'de başlıkta hafta şeridi (ekip görevleri + kilometre taşları, gün tıklanınca `hubDay` listesi).
- Referanslar (ortak projede): tek "Panolar" sekmesi + solda dizin `#rdir` (`renderDir`): Ekip panoları / Benim panolarım.
  Özel pano `board.private`, üzerindeki kartlar da `private` (api.add otomatik), görselleri `<pid>/p/<uid>/<id>` (kartta `ap`; storage RLS yalnızca sahibine).
  Ekibe gönder: sağ tık ya da kartı dizindeki ekip panosunun adına sürükle (`sendToBoard`, dosya `storage.copy` ile ekip klasörüne); "Ekibe aç" panonun tamamını açar. Pano kilidi `board.locked`: adı değişmez, silinemez (`lockBoard`, sağ tık menüsü `boardMenu`). Pano sahibi `board.owner` (oluşturan): ortak projede yalnızca o siler, adını/kilidini/gizliliğini değiştirir (`canMng`; veritabanında `private.guard_board` tetikleyicisi). Diğerleri içeriği düzenler. Sahip "Gizliye al" (`boardToPrivate`) ile panoyu özele çevirir; ekipteki istemcilere broadcast `hide` gider. Dizin gizlenebilir: `setRdir` → `body.rdc`, LS `rdirC`.
- Presence: `presTrack` → `presFlush` (kanal başına yalnızca değişince, ≥1,2 sn arayla; Supabase istemci presence sınırı). Sesli odaya giriş/çıkış ayrıca broadcast `rtc` `join`/`here`/`bye` ile duyurulur.
- Roller `members.role`: `owner` (yönetici) · `admin` (yetkili) · `editor` (üye). Proje ayarları `#shr` (`openShare`, sekmeler Üyeler/Kullanım, `setShTab`).
  Yönetici `set_member_role` ile yetki verir/alır (yalnızca owner). Davet ve davet iptali yönetici + yetkili (`private.is_admin`, `invite_member` `not_admin`);
  çıkarma: yönetici herkesi, yetkili yalnızca üyeleri (`members_delete`). İstemcide `CL.role`, `isAdmin(pid)`.
  Kullanım: `project_usage(p)` (yalnızca yetkili+; depolama, kişi başı, içerik, 30 gün etkinlik, plan doluluğu) + R2 `r2` op `usage` (kova toplamı/proje payı, 10 GB) → `drawUsage`; `purge_deleted(p)` 7 günden eski silinmişleri temizler.
  Sahipsiz panoları ortak projede proje sahibi yönetir.
- Dosyalar Cloudflare R2'de (kova `workshop`): edge function `r2` (secrets `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`)
  imzalı adres verir (`sign` toplu GET / PUT), `del`, `copy`, `purge` (proje sahibi), `migrate` (geliştirici). Yetki `public.r2_can(paths)` = assets RLS ile aynı kural.
  İstemcide `ST` (up/get/del/copy/purge): R2 olmazsa Supabase `assets`'e düşer; okumada R2'de yoksa Supabase'ten (eski sürümler, taşınmamış dosyalar). R2 CORS: Pages + tauri kökenleri.
- Hesap paneli: avatar + ad + e-posta (`acBig`/`acNm`/`acWho`), proje sayıları `acSt`.
- Hata bildirimi (yalnızca geliştirici, `private.devs` → `public.is_dev()`): hesap paneli "Hata bildir" → `#bug` (`openBug`, `sendBug`, bağlam `bugCtx` + son hatalar `ERRS`),
  tablo `public.bug_reports` (status open/fixed/wontfix, `reply`), görseller kova `bugs`. Aynı pencerede "Eski dosyaları R2'ye taşı" (`migrateR2`, `dev_asset_names`).
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
- Zamanlayıcı (`#timerPop`, sekmeler `setTTab`, LS `ttab`): Odak (Pomodoro, `TM`) · Kronometre (`SW`, LS `sw`; `swTask` görevde "süre", `swFinish` → `creditTask` → `task.focus` dk; görevde `est` tahmini dk, `timeBadge`) · Sayaçlar (`CT`, LS `ctimers`, `ctAdd`; büyük sayı `#ctBig` en yakın sayacı gösterir, tıkla-yaz yeni sayaç: tür `#ctModes` Süre (dk:sn, LS `ctLast`) / Bitiş saati (sa:dk, LS `ctAt`, geçmişse ertesi gün; `ctAtMs`), LS `ctMode` (`ctBig`, `ctStart`, `ctPair`); 1 saatten uzun sayaç büyükte sa:dk; bitince `chime`+`notifyAlways`). Kronometre de yuvarlanan rakam (`rollText`). Büyük sayı ve dakika kutuları fare tekerleğiyle değişir.
  Mini sayaç (masaüstü): `miniOpen` → `WebviewWindow("mini", index.html#mini)`, her zaman üstte; sayfa `html.mini` + `#miniRoot`, ana betik `__MINI` ile durur; durum olayı `mini-state`, komut `mini-cmd`; küçültünce/gizleyince otomatik (LS `t_mini`). Tray ipucu `trayTip`. İzinler `capabilities/desktop.json`, pencere listesi `default.json` (`mini`).
  Odak sekmesi: tür `#tmodes` Odak / Mola (tek mola, otomatik molalar da bu süre; `TM.fdur`, LS `sbrk`, `bDur`, `brkKind`); büyük sayı `#tMM`/`#tSS` girişleri yuvarlanan rakamların
  (`rollSet`, `.troll .dg`) üstünde saydam; tıkla-yaz, Enter uygular ve başlatır, tekerlek dk/sn (`tApplyNum`, `tSetSec`). Açılır pencereler düğmenin altına hizalanır (`anchorPop`). Hazır dakika düğmeleri ve "dk/Ayarla" kaldırıldı. Geçmiş son 7 gün; güne gelince o gün nelere odaklanıldığı (`histTip`).
  Odak ekleri: günlük hedef (`fGoal`, LS `fgoal`, `#tgoal` tekerlek/tık) · "aklıma geldi" (`PARK`, LS `park`; odaktayken hızlı ekleme `quickAdd` → `parkAdd`, listesi `#tpark`, odak bitince hatırlatma) ·
  ortam sesi (`AMB`, LS `amb`/`ambv`: yağmur/derin/kafe Web Audio ile üretilir, yalnızca odak sürerken; `ambSync` drawTimer'da) · mola önerileri (`BTIPS`, `BK`, `#brkCard`, OPT `btips`; mini sayaçta kısa hâli) ·
  Geçmiş sekmesi (`drawHist`, son 7 gün: özet, hedef serisi, gün kutuları + `histTip`, saat dağılımı, projeler; `type:"focus"` öğelerinden).
  Spotify (`SPO`): hesap paneli `#spoSec` (`.acspo`) "Bağlan" → PKCE (`spoConnect`; Client ID `app_config.spotify_client_id`). Web: yönlendirme = sayfa adresi, dönüşte `spoBoot`. Masaüstü: `spo_listen` (lib.rs, 127.0.0.1:43117/callback bir kez dinler) + sistem tarayıcısı, kod `spo-code` olayıyla `spoDesk`e; bu adres Spotify panelinde Redirect URI olarak kayıtlı olmalı;
  belirteç LS `spo` (yenileme `spoFetch`). Odak bitince `spoAttach`: son çalınanlar + şu an çalan → focus öğesinde `music:[{id,n,a,img,at}]` (4 dk sonra bir kez daha). Üst çubuğun ortasında `#npChip` (solda piksel yüz `#npFace`: `NPF` çizer, `npfState` idle/sleep(10 dk)/play/change/pause/focus/focus0(müziksiz odak)/error, `npfLoop` rAF yalnızca görünürken; çip üst çubuğun tam ortasında (dikeyde `top:0;bottom:0;margin-block:auto`, yükseklik yüze göre ≥34 px, z 30), `overflow:visible` (alevler taşabilir); `fit(cv,1.5)` hücre (`--npcell`; yüz alev satırları kadar yukarı kaydırılıp ortalanır) = tam sayı ekran pikseli (ölçek %125/%150'de net, `dataset.dpr` değişince yeniden); odak serisi `TM.streak`/`TM.lastF` (`focusStreak`, pencere `streakWin` = 2×odak+mola+20 dk): 2 küçük alev, 3 üç dil+kıvılcım+kaş, 4 büyük alev+kafa büyür (`fire`, `RING4`), yükselişte patlama (`NPFS.burst`) ve bildirim; 4'ten sonra değişmez; çalan yoksa ya da bağlı değilse çip yalnızca yüz (`.npchip.solo`); Ayarlar `#npSw` (LS `npShow`) kapalıysa şarkı adı/süre gizli, yüz yine tepki verir, bağlı değilken pencere "Hesap'tan bağla", renk tema token'larından; tıklayınca `#npPop`: büyük kapak `spoBig`, tam ad/albüm, süre `nppTm`/`nppBar`, son 5 şarkı `npRecent` 30 sn önbellek; absolute, animasyon sınıfı `npin` — `.pop` açılır pencere sınıfıdır, kullanma) (#np; kapaksız, `#npTm` geçen/toplam süre ve alt çizgi `#npBar` saniyede bir yerelde ilerler (`npTick`, şarkı bitince yeniden sorar); bağlı ama çalmıyorsa sönük "şu an çalan yok", tıkla = Spotify'da aç; `spoNow` sayfa görünürken 15 sn'de bir, `drawSpoNow`). Odakta `#tspo` şu an çalan; Geçmiş'te ipucu + "Odakta dinlediklerin".
  Ekip odağı (`TF`): sesli oda panelinde `tfRow` Kapalı/Açık (`tfToggle`, rtc `tf` `tog`); açıkken `tStart`/`tPause`/`tReset`/`tSetSec` → `tfSend` (kalan ms ile, saat farkından bağımsız),
  alanlar `tfApply`. Odak→mola otomatik geçişini yalnızca son işlemi yapan (`TF.lead`) yollar (`finish` → `TF.auto`). Odaya katılana `join` yanıtında durum gider. Odak sekmesinde `#ttf`.
  Odakta: presence `f` (bitiş zamanı ya da 1 = kronometre) → `.hp.fx`; odaktayken `teamNotify` yalnızca acil (bahsetme, özel mesaj).
- Üstte referans (masaüstü, PureRef yerine): pano çubuğu "Üstte aç" → `refOpen` → `WebviewWindow("ref", index.html#ref)` (her zaman üstte, çerçevesiz, saydam).
  Salt görüntü: ana pencere `#world`'ün kopyasını `ref-snap` olayıyla yollar (`refPush`, `syncCards` sonunda `refSched`); görseller Cache Storage `ws-ref`'e yazılır (aynı köken).
  Ref penceresi: kaydır/yakınlaştır, çift tık sığdır, saydamlık (LS `atolye:refop`), sabitle, "tıklamalar arkaya geçsin" (`setIgnoreCursorEvents`; geri Ctrl+Alt+R → `ref-pass`), görünüm LS `atolye:refv:<bid>`.
- Hızlı görev (masaüstü): Ctrl+Alt+Boşluk (global-shortcut → `quick` olayı) → `quickOpen` (`#quick` penceresi) → `quick-add` → `quickAdd`: açık projeye (yoksa projesiz) bugünün görevi, ortak projede özel; `parseTask` (14:30, #etiket).
  Yardımcı pencereler `__WIN` = mini/ref/quick (`__MINI` ile ana betik durur); izinler `capabilities/helpers.json` (Windows'ta `aux` ayrılmış ad, kullanma).
- Paylaş: yerel proje hemen taşınmaz (`shPre`); "Buluta taşı ve davet et" / "Yalnızca buluta taşı" (`shareMove`).
- Ses cihazları ayarlarda (`drawAud`, LS `vMic`/`vOut`); çan ve ses kartları da `vOut`'a çalar. Yazı tipi sabit: Geist + Unbounded (seçenek kaldırıldı).
- Yazı kartı `kind:"text"` (çerçevesiz başlık): `fs` px, genişlik içerikten (`max-content`, `w` ölçülüp saklanır); araç çubuğu "Yazı" / T, köşeden sürükle = boyut (`tresize`), seçiliyken +/−, sağ tık `textSizeMenu` (`TSIZES`), `convText` not ↔ yazı.
- Kart kopyala/kes/yapıştır: Ctrl+C/X → `copySel` (`CLIPB`, sistem panosuna `workshop:<zaman>` işareti; görsel kartta ayrıca PNG `ClipboardItem` ile, başka uygulamalara yapıştırılır), Ctrl+V / boş alana sağ tık → `pasteCards`; başka projeye ya da özel↔ekip panosuna yapıştırırken görseller `blobOf` → `assets.upload` ile yeniden yüklenir.
- Her projede: görsele bağlı not (`card.note`), ses kartı (`kind:"audio"`, Drive linki → `uc?export=download`).
- Drive API anahtarı: `public.app_config` (`drive_key`, okuma herkes, yazma `set_app_config` yalnızca proje sahibi), istemcide LS `dkey`; varsa Drive ses/video `googleapis.com/drive/v3/files/<id>?alt=media&key=` ile akış (büyük dosya uyarısına takılmaz). Ses/video eklenince dosya adı çekilir (`mediaName`: Drive API `fields=name`, ya da linkteki dosya adı); adı boş eski kartlara bir kez yazılır (`fillName`). Anahtar uygulamada görünmez; Supabase panelinden `app_config` tablosuna (`key`=`drive_key`) girilir.

## Giriş ve pencere (masaüstü + web)
- Web (Pages) de aynı giriş ekranını gösterir; hesapsız modda yerel projeler tarayıcıda (öğeler localStorage `wloose`, görseller IndexedDB `ws-local` → `idbAssets`).
- İlk açılış: giriş ekranı `#welcome` (`maybeWelcome`, LS `welcome` = `acct`|`guest`); hesapsız devam = çevrimdışı, bir daha sorulmaz, giriş sağ üstten. Girişsizken ses/bildirim düğmeleri gizli.
- iPad/dokunmatik (`NOKBD`, `html.nokbd`): `kbd`, `.kbonly`, kısayol yardımı gizli; menü etiketlerindeki "· Ctrl+…" (`kbStrip`) ve parantez içi açıklamalar (`kbStrip`), bildirimlerdeki kısayol ipuçları atılır. Panoda iki parmakla dokun = geri al, üç parmak = yinele (`TT`); araç çubuğunda geri al/yinele düğmeleri (`.touchonly`). Uzun basma = sağ tık (dizin satırları ve proje sekmeleri dahil). Web ana ekrana eklenebilir (`manifest.webmanifest`, `apple-touch-icon.png`).
- iPad/dokunmatik ölçek: sayfa yakınlaştırması kapalı (viewport `user-scalable=no`, `gesture*` engelli); arayüz boyutu LS `uiScale` (varsayılan 0.85, ayarlarda `#uiSec`, yalnızca dokunmatik + büyük ekran) viewport `initial/min/max-scale` ile. `html.coarse` sabit gövde + klavye kapanınca `scrollTo(0,0)`. iOS durum çubuğu gizli (`Info.ios.plist` `UIStatusBarHidden`). Pano nokta ızgarası yakınlaştırmaya göre seyrelir (ekranda 16–48 px).
- Sağ tuşla sürükle = pencereyi taşı (PureRef gibi; ana, ref, mini pencerelerde; `cursorPosition` + `setPosition` fiziksel piksel, büyütülmüşken kapalı; kıpırdamadan bırakınca sağ tık menüsü açılır; izinler `default.json` outer/set/cursor-position).
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
  Apple hatası (ITMS-90035): hesap adındaki "Ç" yüzünden Apple'ın bulut imzasının gereksinimi tutmuyor. Çözüm `ios.yml`'de: `tools/asc.py` ile
  geçici Apple Distribution sertifikası + App Store profili (App Store Connect API, anahtar yalnızca makinede), IPA bu kimlikle yeniden imzalanır;
  designated requirement `certificate leaf[subject.OU] = <takım>` (ad yok). Yükleme `altool`; sertifika/profil iş sonunda silinir. Kayıtlar dışarıdan
  okunamadığı için hatalar `::error::` açıklaması olarak yazılır.

## Kullanıcı tercihleri
- Doğrudan, sade çözümler; riskler baştan söylenir.
- Animasyonlar yaylı/"pop" hissinde; tema token'ları `:root` + `[data-theme=light|vivid|ember|sand]` (Kehribar koyu, Kum göz yormayan açık; logo `WMARK` renkleri `--wm`/`--wm2`/`--wmd` ile temaya uyar; açık temaya özel kurallar `:root:is([data-theme="light"],[data-theme="sand"])`, `applyTheme` listesi).
