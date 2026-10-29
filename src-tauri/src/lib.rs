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
            let snip = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyW);
            builder = builder.plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, sc, ev| {
                        if sc == &snip && ev.state() == ShortcutState::Pressed {
                            let _ = app.emit("snip", ());
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
                let _ = _app
                    .global_shortcut()
                    .register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyW));
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
            clip_image
        ])
        .run(tauri::generate_context!());
    if let Err(e) = res {
        let m = e.to_string();
        let wv = m.to_lowercase().contains("webview");
        fatal(&if wv { format!("{WEBVIEW_MSG}\n\nAyrıntı: {m}") } else { format!("Workshop başlatılamadı:\n\n{m}") }, wv);
    }
}
