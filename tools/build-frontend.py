"""web/workshop.html (web ve masaüstü sürümünün ortak kaynağı) → src/index.html

Web sürümü bu dosyanın kendisidir; masaüstü sürümü bu betiğin
ürettiği dosyayı kullanır. Fark: Google Fonts yerine gömülü yazı tipleri,
zip için fflate, ortak projeler için supabase-js ve düzgün bir <head>."""
import pathlib, re

root = pathlib.Path(__file__).resolve().parent.parent
src = (root / "web" / "workshop.html").read_text(encoding="utf-8")

# artifact iskeletini ve <head> dışına düşmüş etiketleri at
body = src.split("<body>", 1)[1]
body = re.sub(r"</body>\s*</html>\s*$", "", body.strip())
body = re.sub(r'<title>.*?</title>\s*', "", body, count=1)
body = re.sub(r'<meta name="claude:[^>]*>\s*', "", body)
body = re.sub(r'<link rel="preconnect"[^>]*>\s*', "", body)
body = re.sub(r'<link rel="stylesheet" href="https://fonts\.googleapis\.com[^>]*>\s*', "", body)
assert "fonts.googleapis" not in body, "Google Fonts bağlantısı kaldı"

head = """<!doctype html>
<html lang="tr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no,viewport-fit=cover">
<title>Workshop</title>
<link rel="icon" href="favicon.svg">
<link rel="apple-touch-icon" href="apple-touch-icon.png">
<link rel="manifest" href="manifest.webmanifest">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black">
<meta name="apple-mobile-web-app-title" content="Workshop">
<meta name="theme-color" content="#0D0E10">
<link rel="stylesheet" href="fonts.css">
<style>html{box-sizing:border-box;background:#0D0E10}body{margin:0;padding:0}img{max-width:100%}[hidden]:not([hidden=until-found i]){display:none!important}</style>
<script src="vendor/fflate.js"></script>
<script src="vendor/supabase.js"></script>
</head>
<body>
"""
(root / "src" / "index.html").write_text(head + body + "\n</body>\n</html>\n", encoding="utf-8")
print("src/index.html written")
