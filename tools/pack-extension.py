"""claude-extension/ → claude-extension/workshop.mcpb (Claude masaüstü uzantısı; zip arşivi).

manifest.json ya da server/index.js değişince yeniden çalıştır ve .mcpb dosyasını da gönder:
uygulamadaki "Claude uzantısını indir" bağlantısı depodaki bu dosyayı indirir."""
import pathlib, zipfile

root = pathlib.Path(__file__).resolve().parent.parent / "claude-extension"
out = root / "workshop.mcpb"
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for rel in ("manifest.json", "server/index.js"):
        info = zipfile.ZipInfo(rel, date_time=(2026, 1, 1, 0, 0, 0))  # sabit tarih: içerik aynıysa dosya da aynı
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o644 << 16
        z.writestr(info, (root / rel).read_bytes())
print(out, out.stat().st_size, "bayt")
