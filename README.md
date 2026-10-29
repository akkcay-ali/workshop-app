# Workshop

Kişisel planlama ve referans panosu: projeler, günlük görevler, takvim, odak
zamanlayıcısı ve serbest yerleşimli moodboard'lar. Masaüstü sürümü Tauri 2 ile
paketlenir; aynı arayüz web sürümü olarak da çalışır.

## Kurulum (Windows)

1. Bu deponun **Releases** sayfasından en yeni `Workshop_…_x64-setup.exe` dosyasını indir.
2. Çalıştır. Windows SmartScreen "tanınmayan uygulama" diyebilir: **Ek bilgi → Yine de çalıştır**.
   (Uygulama imzalı değil; kişisel kullanım için sorun değil.)
3. Kurulum sadece senin kullanıcı hesabına yapılır, yönetici izni istemez.

Her `main` gönderiminde GitHub yeni kurulum dosyasını kendisi derler (Actions → *Windows derlemesi*).

**Güncellemeler:** 0.2.0'dan itibaren uygulama açılışta yeni sürüm olup olmadığına bakar; varsa üst çubukta
**Güncelle** düğmesi çıkar, tek tıkla indirip kurar ve yeniden açılır. Ayarlar → *Güncellemeleri denetle* ile elle de bakılabilir.
Güncelleme paketleri imzalıdır; uygulama sadece bu deponun imza anahtarıyla imzalanmış sürümleri kurar.

## Dosyalar

- Her proje tek bir **`.workshop`** dosyasıdır. İçinde proje verisi (`project.json`) ve
  panodaki görseller (`images/`) bulunur; aslında bir zip arşividir.
- Varsayılan klasör: **Belgeler\Workshop**. Ayarlar → *Proje klasörü* bölümünden değiştirilebilir
  (ör. OneDrive içindeki bir klasör seçilirse dosyalar bulutta da yedeklenir).
- Değişiklikler otomatik kaydedilir. Sekmedeki bulut ikonu yeşilse dosya güncel, griyse kaydediliyor.
- `.workshop` dosyasına çift tıklamak onu Workshop'ta açar. Kütüphane dışındaki dosyalar
  yerinde düzenlenir ve sonraki açılışlarda da listelenir.
- Silinen projeler Geri Dönüşüm Kutusu'na gider.
- Panodan silinen görseller bir sonraki kayıtta dosyadan da çıkar; eski görseller saklanmaz.

## Web sürümünden taşıma

Web sürümünde Ayarlar → **Yedek indir** ile bir `.json` yedeği al, masaüstü uygulamasında
Ayarlar → **Web yedeğini içe aktar** ile aç. Her proje ayrı bir `.workshop` dosyası olur.

## Proje yapısı

```
web/workshop.html        Ortak kaynak (web ve masaüstü arayüzü tek dosyada)
tools/build-frontend.py  web/workshop.html → src/index.html (gömülü yazı tipleri, fflate)
src/                     Masaüstü arayüzü: index.html, fonts/, vendor/fflate.js
src-tauri/               Rust kabuğu: dosya okuma/yazma, kütüphane klasörü, çift tıkla açma
.github/workflows/       Windows (otomatik) ve macOS (elle) derlemeleri
```

Arayüz değişiklikleri **`web/workshop.html`** üzerinde yapılır, ardından
`python tools/build-frontend.py` çalıştırılır. Masaüstüne özgü kod bu dosyada
`window.__TAURI__` varken devreye girer (dosya modu); web sürümünde etkisizdir.

## Yerelde geliştirme

Gerekenler: Node 22, Rust (stable), Windows'ta *Microsoft C++ Build Tools*.

```
npm ci
python tools/build-frontend.py
npx tauri dev        # geliştirme penceresi
npx tauri build      # kurulum dosyası (src-tauri/target/release/bundle/)
```

## iPad sürümü (Mac gerekir)

Tauri 2 iOS'u destekler; derleme yalnızca Mac'te Xcode ile yapılabilir.

1. Mac'e Xcode, Node 22 ve Rust kur; `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`.
2. Depoyu klonla, `npm ci`, `python3 tools/build-frontend.py`.
3. `npx tauri ios init` (bir kez) → `npx tauri ios dev` ile iPad'e/simülatöre yükle.
4. Kalıcı kurulum için Apple Developer hesabı gerekir (ücretsiz hesapla yüklenen uygulama 7 günde bir yenilenmelidir).

iPad'de dosyalar uygulamanın Belgeler klasöründe tutulur; iCloud Drive ile PC arasında
paylaşım ayrı bir adım olarak eklenecek.
