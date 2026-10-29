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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // Uygulama zaten açıkken başka bir .workshop dosyasına çift tıklanırsa
    // ikinci bir pencere açmak yerine dosyayı açık olan pencereye gönder.
    #[cfg(desktop)]
    {
        use tauri::Emitter;
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let files = workshop_args(&args);
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
            if !files.is_empty() {
                let _ = app.emit("open-files", files);
            }
        }));
        // Kendi kendini güncelleme: GitHub sürümlerindeki imzalı latest.json'u denetler
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init());
    }

    builder
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
            initial_files
        ])
        .run(tauri::generate_context!())
        .expect("Workshop başlatılamadı");
}
