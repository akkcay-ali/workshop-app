//! Workshop masaüstü kabuğu.
//! Arayüz `src/` altındaki web kodudur; burası sadece dosya işlerini yapar:
//! .workshop dosyalarını okuma/yazma, kütüphane klasörü, çöp kutusuna taşıma
//! ve çift tıklanan dosyaları arayüze iletme.

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct FileInfo {
    path: String,
    name: String,
    modified: u64,
    size: u64,
}

/// Windows `canonicalize` çıktısındaki `\\?\` önekini atar.
fn clean(p: PathBuf) -> String {
    let s = p.to_string_lossy().to_string();
    s.strip_prefix(r"\\?\").map(|x| x.to_string()).unwrap_or(s)
}

/// Komut satırı argümanlarından .workshop dosyalarını seçer (çift tıklama / "Birlikte aç").
fn workshop_args(args: &[String]) -> Vec<String> {
    args.iter()
        .skip(1)
        .filter(|a| a.to_lowercase().ends_with(".workshop"))
        .map(|a| fs::canonicalize(a).map(clean).unwrap_or_else(|_| a.clone()))
        .collect()
}

fn err<E: std::fmt::Display>(path: &str) -> impl Fn(E) -> String + '_ {
    move |e| format!("{path}: {e}")
}

/// Varsayılan proje klasörü: Belgeler\Workshop
#[tauri::command]
fn library_dir(app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .document_dir()
        .map_err(|e| e.to_string())?
        .join("Workshop");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(clean(dir))
}

#[tauri::command]
fn ensure_dir(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(err(&path))
}

/// Klasördeki .workshop dosyaları (alt klasörlere inmeden).
#[tauri::command]
fn list_workshops(dir: String) -> Result<Vec<FileInfo>, String> {
    let mut out = Vec::new();
    let rd = match fs::read_dir(&dir) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(format!("{dir}: {e}")),
    };
    for entry in rd.flatten() {
        let p = entry.path();
        let is_ws = p
            .extension()
            .map(|x| x.to_string_lossy().eq_ignore_ascii_case("workshop"))
            .unwrap_or(false);
        if !is_ws || !p.is_file() {
            continue;
        }
        let meta = entry.metadata().ok();
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        out.push(FileInfo {
            name: p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            path: clean(p.clone()),
            modified,
            size: meta.map(|m| m.len()).unwrap_or(0),
        });
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

/// Dosyayı ham bayt olarak döndürür (JSON'a çevirmeden, büyük dosyalarda hızlı).
#[tauri::command]
fn read_bytes(path: String) -> Result<tauri::ipc::Response, String> {
    let data = fs::read(&path).map_err(err(&path))?;
    Ok(tauri::ipc::Response::new(data))
}

/// Ham baytları güvenli biçimde yazar: önce geçici dosyaya, sonra yerine taşır.
/// Yazma yarıda kesilirse eski dosya bozulmaz. Yol, `path` başlığında URL-kodlu gelir.
#[tauri::command]
fn write_bytes(request: tauri::ipc::Request) -> Result<(), String> {
    let tauri::ipc::InvokeBody::Raw(data) = request.body() else {
        return Err("ham veri bekleniyordu".into());
    };
    let raw = request
        .headers()
        .get("path")
        .ok_or("dosya yolu eksik")?
        .to_str()
        .map_err(|e| e.to_string())?;
    let path = urlencoding::decode(raw).map_err(|e| e.to_string())?.into_owned();
    let target = PathBuf::from(&path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(err(&path))?;
    }
    let tmp = PathBuf::from(format!("{path}.tmp"));
    fs::write(&tmp, data).map_err(err(&path))?;
    fs::rename(&tmp, &target).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{path}: {e}")
    })
}

#[tauri::command]
fn exists(path: String) -> bool {
    Path::new(&path).exists()
}

/// Yeniden adlandırır; hedef zaten varsa üzerine yazmaz.
#[tauri::command]
fn rename_file(from: String, to: String) -> Result<(), String> {
    if Path::new(&to).exists() {
        return Err(format!("{to}: zaten var"));
    }
    fs::rename(&from, &to).map_err(err(&from))
}

/// Silinen projeler kalıcı silinmez, Geri Dönüşüm Kutusu'na gider.
#[tauri::command]
fn trash_file(path: String) -> Result<(), String> {
    #[cfg(desktop)]
    {
        trash::delete(&path).map_err(err(&path))
    }
    #[cfg(not(desktop))]
    {
        fs::remove_file(&path).map_err(err(&path))
    }
}

/// Uygulama bir .workshop dosyasına çift tıklanarak açıldıysa o dosyalar.
#[tauri::command]
fn initial_files() -> Vec<String> {
    workshop_args(&std::env::args().collect::<Vec<_>>())
}

/// Spotify bağlantısı (masaüstü): Spotify dönüşü http://127.0.0.1:43117/callback adresine gelir.
/// Bir kez dinler (en çok 5 dk), tarayıcıya kısa bir sayfa döner, sorgu dizesini `spo-code` olayıyla arayüze yollar.
/// Port doluysa (önceki deneme hâlâ dinliyorsa) hata döner; arayüz bunu yok sayar, o dinleyici yakalar.
#[tauri::command]
fn spo_listen(app: tauri::AppHandle) -> Result<(), String> {
    use std::io::{Read, Write};
    use tauri::Emitter;
    let l = std::net::TcpListener::bind("127.0.0.1:43117").map_err(|e| e.to_string())?;
    l.set_nonblocking(true).map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < 300 {
            match l.accept() {
                Ok((mut s, _)) => {
                    let _ = s.set_nonblocking(false);
                    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                    let mut buf = [0u8; 8192];
                    let n = s.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    let path = req.lines().next().unwrap_or("").split_whitespace().nth(1).unwrap_or("").to_string();
                    if !path.starts_with("/callback") {
                        let _ = s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                        continue;
                    }
                    let body = "<!doctype html><meta charset=utf-8><title>Workshop</title><body style=\"font-family:system-ui,sans-serif;background:#111216;color:#e8eaee;display:grid;place-items:center;height:100vh;margin:0\"><div style=\"text-align:center\"><b>Spotify bağlandı</b><p style=\"color:#9aa0ac\">Bu sekmeyi kapatıp Workshop'a dönebilirsin.</p></div></body>";
                    let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
                    let _ = s.write_all(resp.as_bytes());
                    let q = path.splitn(2, '?').nth(1).unwrap_or("").to_string();
                    let _ = app.emit("spo-code", q);
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(std::time::Duration::from_millis(150)),
                Err(_) => break,
            }
        }
    });
    Ok(())
}

/// Claude bağlantısı (masaüstü): Claude masaüstü uygulamasına kurulan Workshop uzantısı (`claude-extension/`)
/// bu bilgisayardaki 127.0.0.1:43118 adresine istek atar; istek `claude-req` olayıyla arayüze gider,
/// arayüz `bridge_reply` ile yanıtlar. Yalnızca bu makineden erişilir, her açılışta yenilenen anahtar ister.
/// Adres ve anahtar `~/.workshop/claude-bridge.json` dosyasına yazılır (uzantı oradan okur).
mod bridge {
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{mpsc, Mutex, OnceLock};
    use std::time::Duration;
    use tauri::{AppHandle, Emitter, Manager};

    const PORT: u16 = 43118;
    const MAX_BODY: usize = 48 * 1024 * 1024;
    static ON: AtomicBool = AtomicBool::new(false);
    static STARTED: AtomicBool = AtomicBool::new(false);
    static SEQ: AtomicU64 = AtomicU64::new(1);
    static TOKEN: OnceLock<String> = OnceLock::new();
    static PENDING: OnceLock<Mutex<HashMap<u64, mpsc::Sender<String>>>> = OnceLock::new();

    #[derive(Clone, serde::Serialize)]
    struct Req {
        id: u64,
        body: String,
    }

    fn pending() -> &'static Mutex<HashMap<u64, mpsc::Sender<String>>> {
        PENDING.get_or_init(|| Mutex::new(HashMap::new()))
    }

    /// Tahmin edilemeyen anahtar: işletim sisteminin rastgele tohumladığı SipHash anahtarlarından.
    fn token() -> &'static str {
        TOKEN.get_or_init(|| {
            use std::hash::{BuildHasher, Hasher};
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            (0..4u64)
                .map(|i| {
                    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
                    h.write_u64(t ^ i);
                    format!("{:016x}", h.finish())
                })
                .collect()
        })
    }

    fn info_path(app: &AppHandle) -> Result<PathBuf, String> {
        Ok(app.path().home_dir().map_err(|e| e.to_string())?.join(".workshop").join("claude-bridge.json"))
    }

    fn respond(s: &mut TcpStream, status: &str, body: &str) {
        let _ = s.write_all(
            format!("HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes(),
        );
        let _ = s.write_all(body.as_bytes());
    }

    fn handle(mut s: TcpStream, app: AppHandle) {
        let _ = s.set_read_timeout(Some(Duration::from_secs(20)));
        let mut buf: Vec<u8> = Vec::with_capacity(8192);
        let mut tmp = [0u8; 16384];
        let head_end = loop {
            if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break p + 4;
            }
            if buf.len() > 65536 {
                return respond(&mut s, "431 Request Header Fields Too Large", "{}");
            }
            match s.read(&mut tmp) {
                Ok(0) | Err(_) => return,
                Ok(n) => buf.extend_from_slice(&tmp[..n]),
            }
        };
        let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
        let mut lines = head.lines();
        let first = lines.next().unwrap_or("");
        let mut auth = String::new();
        let mut len = 0usize;
        let mut origin = false;
        for l in lines {
            if let Some((k, v)) = l.split_once(':') {
                match k.trim().to_ascii_lowercase().as_str() {
                    "authorization" => auth = v.trim().to_string(),
                    "content-length" => len = v.trim().parse().unwrap_or(0),
                    "origin" => origin = true,
                    _ => {}
                }
            }
        }
        if !first.starts_with("POST /rpc ") {
            return respond(&mut s, "404 Not Found", "{}");
        }
        // Tarayıcıdaki bir sayfa buraya istek atamasın: tarayıcılar Origin yollar, uzantı yollamaz
        if origin || auth != format!("Bearer {}", token()) {
            return respond(&mut s, "401 Unauthorized", r#"{"ok":false,"error":"yetkisiz"}"#);
        }
        if !ON.load(Ordering::SeqCst) {
            return respond(&mut s, "503 Service Unavailable", r#"{"ok":false,"error":"Claude bağlantısı Workshop ayarlarında kapalı"}"#);
        }
        if len > MAX_BODY {
            return respond(&mut s, "413 Payload Too Large", r#"{"ok":false,"error":"istek çok büyük"}"#);
        }
        let mut body = buf[head_end..].to_vec();
        while body.len() < len {
            match s.read(&mut tmp) {
                Ok(0) | Err(_) => return,
                Ok(n) => body.extend_from_slice(&tmp[..n]),
            }
        }
        body.truncate(len);
        let id = SEQ.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();
        pending().lock().unwrap().insert(id, tx);
        let _ = app.emit_to("main", "claude-req", Req { id, body: String::from_utf8_lossy(&body).to_string() });
        match rx.recv_timeout(Duration::from_secs(120)) {
            Ok(out) => respond(&mut s, "200 OK", &out),
            Err(_) => {
                pending().lock().unwrap().remove(&id);
                respond(&mut s, "504 Gateway Timeout", r#"{"ok":false,"error":"Workshop yanıt vermedi"}"#)
            }
        }
    }

    /// Dinleyiciyi (bir kez) başlatır, bağlantıyı açar ve adres/anahtar dosyasını yazar.
    #[tauri::command]
    pub fn bridge_start(app: AppHandle) -> Result<(), String> {
        if cfg!(mobile) {
            return Err("yalnızca masaüstü".into());
        }
        if !STARTED.load(Ordering::SeqCst) {
            let l = TcpListener::bind(("127.0.0.1", PORT)).map_err(|e| format!("{PORT} numaralı port açılamadı: {e}"))?;
            STARTED.store(true, Ordering::SeqCst);
            let a = app.clone();
            std::thread::spawn(move || {
                for s in l.incoming().flatten() {
                    let a = a.clone();
                    std::thread::spawn(move || handle(s, a));
                }
            });
        }
        let p = info_path(&app)?;
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        let info = serde_json::json!({ "port": PORT, "token": token(), "pid": std::process::id() });
        std::fs::write(&p, info.to_string()).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
        }
        ON.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Bağlantıyı kapatır: istekler reddedilir, adres/anahtar dosyası silinir.
    #[tauri::command]
    pub fn bridge_stop(app: AppHandle) {
        ON.store(false, Ordering::SeqCst);
        if let Ok(p) = info_path(&app) {
            let _ = std::fs::remove_file(p);
        }
    }

    /// Arayüzün `claude-req` isteğine yanıtı (JSON metni).
    #[tauri::command]
    pub fn bridge_reply(id: u64, body: String) {
        if let Some(tx) = pending().lock().unwrap().remove(&id) {
            let _ = tx.send(body);
        }
    }
}

/// Ekran alıntısı: sistemin kendi alıntı aracını açar (Windows: Ekran Alıntısı, macOS: screencapture).
/// Seçilen bölge panoya (clipboard) kopyalanır; arayüz `clip_seq`/`clip_image` ile alır.
#[tauri::command]
fn snip_start() -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg("ms-screenclip:")
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("screencapture")
            .args(["-i", "-c"])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err("Bu sistemde ekran alıntısı desteklenmiyor".into())
}

/// Panonun değişim sayacı (Windows). Diğer sistemlerde 0: arayüz içerik karşılaştırır.
#[tauri::command]
fn clip_seq() -> u32 {
    #[cfg(windows)]
    {
        return unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() };
    }
    #[allow(unreachable_code)]
    0
}

/// Panodaki görsel PNG olarak (yoksa boş).
#[tauri::command]
fn clip_image() -> tauri::ipc::Response {
    #[cfg(desktop)]
    {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(img) = cb.get_image() {
                let mut out = Vec::new();
                {
                    let mut enc = png::Encoder::new(&mut out, img.width as u32, img.height as u32);
                    enc.set_color(png::ColorType::Rgba);
                    enc.set_depth(png::BitDepth::Eight);
                    if let Ok(mut w) = enc.write_header() {
                        let _ = w.write_image_data(&img.bytes);
                    }
                }
                return tauri::ipc::Response::new(out);
            }
        }
    }
    tauri::ipc::Response::new(Vec::new())
}

/// Açılış hatası: sessizce kapanmak yerine nedenini söyleyen bir Windows penceresi.
/// `webview` doğruysa WebView2 indirme sayfasını açmayı önerir.
#[cfg(windows)]
fn fatal(msg: &str, webview: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONERROR, MB_OK, MB_YESNO};
    let w = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let text = w(msg);
    let title = w("Workshop açılamadı");
    let flags = if webview { MB_YESNO | MB_ICONERROR } else { MB_OK | MB_ICONERROR };
    let r = unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), flags) };
    if webview && r == IDYES {
        let _ = std::process::Command::new("explorer.exe")
            .arg("https://developer.microsoft.com/microsoft-edge/webview2/")
            .spawn();
    }
}
#[cfg(not(windows))]
fn fatal(msg: &str, _webview: bool) {
    eprintln!("Workshop açılamadı: {msg}");
}

const WEBVIEW_MSG: &str = "Bu bilgisayarda Microsoft Edge WebView2 bulunamadı ya da çalışmıyor. Workshop'un arayüzü için gerekli.\n\nİndirme sayfasını açayım mı? Oradan \"Evergreen Standalone Installer – x64\" dosyasını indirip kur, sonra Workshop'u yeniden aç.\n\nİnternetsiz kurulum için Workshop sürümler sayfasındaki \"offline-setup\" dosyasını da kullanabilirsin.";

/// Pencereyi gizlendiği yerden (simge alanı) geri getir.
#[cfg(desktop)]
fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Beklenmeyen bir çökme olursa en azından nedenini göster
    std::panic::set_hook(Box::new(|info| {
        fatal(&format!("Beklenmeyen bir hata oluştu:\n\n{info}"), false);
    }));
    #[cfg(windows)]
    {
        if tauri::webview_version().is_err() {
            fatal(WEBVIEW_MSG, true);
            return;
        }
    }

    let mut builder = tauri::Builder::default();

    // Uygulama zaten açıkken başka bir .workshop dosyasına çift tıklanırsa
    // ikinci bir pencere açmak yerine dosyayı açık olan pencereye gönder.
    #[cfg(desktop)]
    {
        use tauri::Emitter;
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let files = workshop_args(&args);
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
            if !files.is_empty() {
                let _ = app.emit("open-files", files);
            }
        }));
        // Ctrl+Alt+W: uygulama arkadayken bile ekran alıntısını panoya ekle
        {
            use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
            // Ctrl+Alt+Boşluk: hızlı görev kutusu · Ctrl+Alt+R: referans penceresinde "tıklamalar arkaya geçsin"i kapat
            let snip = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyW);
            let quick = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
            let refp = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyR);
            // sesli oda: Ctrl+Alt+M mikrofon, Ctrl+Alt+D kulaklık, Ctrl+Alt+V basılı tut = konuş
            let vmute = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyM);
            let vdeaf = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyD);
            let vptt = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV);
            builder = builder.plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, sc, ev| {
                        if sc == &vptt {
                            let _ = app.emit("vkey", ("ptt", ev.state() == ShortcutState::Pressed));
                            return;
                        }
                        if ev.state() != ShortcutState::Pressed {
                            return;
                        }
                        if sc == &vmute {
                            let _ = app.emit("vkey", ("mute", true));
                        } else if sc == &vdeaf {
                            let _ = app.emit("vkey", ("deaf", true));
                        } else if sc == &snip {
                            let _ = app.emit("snip", ());
                        } else if sc == &quick {
                            let _ = app.emit("quick", ());
                        } else if sc == &refp {
                            let _ = app.emit("ref-pass", ());
                        }
                    })
                    .build(),
            );
        }
        // Kendi kendini güncelleme: GitHub sürümlerindeki imzalı latest.json'u denetler
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init());
    }

    let res = builder
        .setup(|_app| {
            // kısayol başka bir uygulamada kullanılıyorsa sessizce geç
            // Simge alanı (saatin yanı): × pencereyi buraya gizler. Tık = aç, sağ tık = menü.
            #[cfg(desktop)]
            {
                use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
                use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
                use tauri::Emitter;
                let open = MenuItem::with_id(_app, "show", "Workshop'u aç", true, None::<&str>)?;
                let quit = MenuItem::with_id(_app, "quit", "Çıkış", true, None::<&str>)?;
                let sep = PredefinedMenuItem::separator(_app)?;
                let menu = Menu::with_items(_app, &[&open, &sep, &quit])?;
                let mut tray = TrayIconBuilder::with_id("main")
                    .tooltip("Workshop")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, ev| match ev.id().as_ref() {
                        "show" => show_main(app),
                        "quit" => {
                            // arayüz bekleyen kayıtları bitirip kapanır; yanıt vermezse 6 sn sonra zorla çık
                            let _ = app.emit("tray-quit", ());
                            let h = app.clone();
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_secs(6));
                                h.exit(0);
                            });
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, ev| {
                        if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                            show_main(tray.app_handle());
                        }
                    });
                if let Some(icon) = _app.default_window_icon() {
                    tray = tray.icon(icon.clone());
                }
                tray.build(_app)?;
            }
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
                for code in [Code::KeyW, Code::Space, Code::KeyR, Code::KeyM, Code::KeyD, Code::KeyV] {
                    let _ = _app
                        .global_shortcut()
                        .register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), code));
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            library_dir,
            ensure_dir,
            list_workshops,
            read_bytes,
            write_bytes,
            exists,
            rename_file,
            trash_file,
            initial_files,
            snip_start,
            clip_seq,
            clip_image,
            spo_listen,
            bridge::bridge_start,
            bridge::bridge_stop,
            bridge::bridge_reply
        ])
        .run(tauri::generate_context!());
    if let Err(e) = res {
        let m = e.to_string();
        let wv = m.to_lowercase().contains("webview");
        fatal(&if wv { format!("{WEBVIEW_MSG}\n\nAyrıntı: {m}") } else { format!("Workshop başlatılamadı:\n\n{m}") }, wv);
    }
}
