#!/usr/bin/env node
// Workshop ↔ Claude: stdio üzerinden MCP sunucusu. Bağımlılığı yoktur.
// İstekleri bu bilgisayardaki Workshop uygulamasına (127.0.0.1) iletir; adres ve anahtar
// ~/.workshop/claude-bridge.json dosyasından okunur (Workshop her açılışta yeniler).
"use strict";
const fs = require("fs"), os = require("os"), path = require("path"), http = require("http");

const INFO = process.env.WORKSHOP_BRIDGE_FILE || path.join(os.homedir(), ".workshop", "claude-bridge.json");
const OFF = "Workshop'a ulaşılamadı. Workshop masaüstü uygulaması açık olmalı ve Ayarlar › Claude bağlantısı açık olmalı. Kullanıcıdan bunu kontrol etmesini iste.";

const card = {
  type: "object",
  properties: {
    kind: { type: "string", enum: ["note", "text", "link", "image"], description: "note = sticky note with body text; text = large frameless heading; link = URL card; image = picture. Defaults: image if path/data given, link if only url, else note." },
    text: { type: "string", description: "Note body, heading text, link title, or (for images) a caption stored as the image's note." },
    url: { type: "string", description: "For link: the page URL. For image: a direct image URL (downloaded into the project; if the site blocks download it is attached as a linked image)." },
    path: { type: "string", description: "For image: absolute path of an image file on this computer (png, jpg, webp, gif, svg)." },
    data: { type: "string", description: "For image: base64 image bytes (use with mime)." },
    mime: { type: "string", description: "MIME type for data, e.g. image/png." },
    size: { type: "number", description: "For text: font size in px (default 56; 96 = title, 36 = large, 24 = normal)." },
    tags: { type: "array", items: { type: "string" }, description: "Optional tags (searchable as #tag in Workshop)." },
    x: { type: "number", description: "Optional absolute board X. Omit x/y to let Workshop lay the cards out in a tidy grid." },
    y: { type: "number", description: "Optional absolute board Y." },
    w: { type: "number", description: "Optional width in board px (image default 280, note 240)." }
  }
};
const board = { type: "string", description: "Board name or id. Omit to use the board currently open in Workshop (or the last opened one)." };

const TOOLS = [
  { name: "workshop_status", description: "Shows what is open in the user's Workshop desktop app: the current project and its boards (name, id, card count, which one is open). Call this first. Workshop only exposes the project whose tab is currently open.", inputSchema: { type: "object", properties: {} }, annotations: { readOnlyHint: true } },
  { name: "workshop_list_cards", description: "Lists the cards on a board with id, kind (image, note, text, link, frame, video, audio, palette), text, tags and geometry. Board coordinates are in px, x grows right, y grows down; each card has x, y (top-left), w and h. Image contents are not included; use workshop_view_cards to look at them.", inputSchema: { type: "object", properties: { board } }, annotations: { readOnlyHint: true } },
  { name: "workshop_view_cards", description: "Returns small previews (max 512 px) of up to 8 image cards so you can see what the references show before grouping or arranging them.", inputSchema: { type: "object", properties: { ids: { type: "array", items: { type: "string" }, maxItems: 8, description: "Image card ids from workshop_list_cards." } }, required: ["ids"] }, annotations: { readOnlyHint: true } },
  { name: "workshop_add_cards", description: "Adds notes, headings, links and images to a board in the open Workshop project. Without x/y the new cards are placed in a grid to the right of the existing content, so nothing overlaps. Pass title to wrap the new cards in a named frame (good for 'references Claude gathered for X'). The user can undo the whole addition in Workshop. Up to 60 cards per call.", inputSchema: { type: "object", properties: { board, title: { type: "string", description: "Optional frame title drawn around the added cards." }, cards: { type: "array", items: card, minItems: 1, maxItems: 60 }, x: { type: "number", description: "Optional board X of the block's top-left corner." }, y: { type: "number", description: "Optional board Y of the block's top-left corner." } }, required: ["cards"] } },
  { name: "workshop_move_cards", description: "Moves and/or resizes existing cards to arrange a board. Read positions with workshop_list_cards first, compute a layout that does not overlap (leave about 24 px gaps), then send all moves in one call so the user can undo it in one step. Locked cards are skipped. Image height follows width (aspect ratio is kept).", inputSchema: { type: "object", properties: { moves: { type: "array", minItems: 1, maxItems: 400, items: { type: "object", properties: { id: { type: "string" }, x: { type: "number" }, y: { type: "number" }, w: { type: "number", description: "New width (optional)." }, h: { type: "number", description: "New height, frames only (optional)." } }, required: ["id"] } } }, required: ["moves"] } },
  { name: "workshop_create_board", description: "Creates a new board in the open project and returns its id. In a shared (team) project the board is private to the user unless private is false.", inputSchema: { type: "object", properties: { name: { type: "string" }, private: { type: "boolean" } }, required: ["name"] } }
];

function call(tool, args) {
  return new Promise((resolve, reject) => {
    let info;
    try { info = JSON.parse(fs.readFileSync(INFO, "utf8")); } catch (_) { return reject(new Error(OFF)); }
    const body = Buffer.from(JSON.stringify({ tool, args }), "utf8");
    const req = http.request({ host: "127.0.0.1", port: info.port, path: "/rpc", method: "POST", timeout: 130000,
      headers: { "Content-Type": "application/json", "Content-Length": body.length, Authorization: "Bearer " + info.token } }, res => {
      const chunks = [];
      res.on("data", c => chunks.push(c));
      res.on("end", () => {
        let out = null;
        try { out = JSON.parse(Buffer.concat(chunks).toString("utf8")); } catch (_) {}
        if (res.statusCode === 401) return reject(new Error(OFF));
        if (!out || !out.ok) return reject(new Error((out && out.error) || "Workshop yanıt vermedi (" + res.statusCode + ")"));
        resolve(out.result);
      });
    });
    req.on("timeout", () => req.destroy(new Error("Workshop yanıt vermedi")));
    req.on("error", e => reject(new Error(e.code === "ECONNREFUSED" || e.code === "ECONNRESET" ? OFF : e.message)));
    req.end(body);
  });
}

// Web görsellerini burada indir: uygulamanın içindeki sayfa çoğu siteden indiremez (CORS), Node indirebilir.
async function fetchImage(url) {
  const ac = new AbortController(), t = setTimeout(() => ac.abort(), 20000);
  try {
    const r = await fetch(url, { signal: ac.signal, redirect: "follow", headers: { "User-Agent": "Mozilla/5.0 Workshop" } });
    if (!r.ok) return null;
    const mime = (r.headers.get("content-type") || "").split(";")[0].trim();
    if (!/^image\//.test(mime)) return null;
    const buf = Buffer.from(await r.arrayBuffer());
    if (!buf.length || buf.length > 25 * 1024 * 1024) return null;
    return { data: buf.toString("base64"), mime };
  } catch (_) { return null; } finally { clearTimeout(t); }
}

async function run(name, args) {
  args = args || {};
  if (name === "workshop_status") return text(await call("status", {}));
  if (name === "workshop_list_cards") return text(await call("list_cards", args));
  if (name === "workshop_create_board") return text(await call("create_board", args));
  if (name === "workshop_move_cards") return text(await call("move_cards", args));
  if (name === "workshop_view_cards") {
    const r = await call("view_cards", args), content = [];
    for (const im of r.images || []) content.push({ type: "text", text: "card " + im.id }, { type: "image", data: im.data, mimeType: im.mime });
    if ((r.skipped || []).length || !content.length) content.push({ type: "text", text: JSON.stringify({ skipped: r.skipped || [] }) });
    return { content };
  }
  if (name === "workshop_add_cards") {
    const cards = [];
    for (const c of Array.isArray(args.cards) ? args.cards : []) {
      const k = Object.assign({}, c), isImg = k.kind === "image" || (!k.kind && /\.(png|jpe?g|webp|gif|avif)(\?|#|$)/i.test(k.url || ""));
      if (isImg && k.url && !k.path && !k.data && /^https?:\/\//i.test(k.url)) {
        k.kind = "image";
        const got = await fetchImage(k.url);
        if (got) Object.assign(k, got);
      }
      cards.push(k);
    }
    return text(await call("add_cards", Object.assign({}, args, { cards })));
  }
  throw new Error("bilinmeyen araç: " + name);
}
const text = v => ({ content: [{ type: "text", text: typeof v === "string" ? v : JSON.stringify(v) }] });

const send = m => process.stdout.write(JSON.stringify(m) + "\n");
async function handle(m) {
  if (!m || m.jsonrpc !== "2.0" || typeof m.method !== "string") return;
  const reply = result => m.id !== undefined && send({ jsonrpc: "2.0", id: m.id, result });
  const fail = (code, message) => m.id !== undefined && send({ jsonrpc: "2.0", id: m.id, error: { code, message } });
  if (m.method === "initialize") return reply({ protocolVersion: (m.params && m.params.protocolVersion) || "2025-06-18", capabilities: { tools: {} }, serverInfo: { name: "workshop", version: "0.1.0" },
    instructions: "Workshop is the user's planning and reference-board app. Tools act on the project currently open in the Workshop desktop app. Call workshop_status first. Nothing can be deleted through these tools." });
  if (m.method === "ping") return reply({});
  if (m.method === "tools/list") return reply({ tools: TOOLS });
  if (m.method === "tools/call") {
    try { return reply(await run(m.params && m.params.name, m.params && m.params.arguments)); }
    catch (e) { return reply({ content: [{ type: "text", text: String((e && e.message) || e) }], isError: true }); }
  }
  if (m.method === "resources/list") return reply({ resources: [] });
  if (m.method === "prompts/list") return reply({ prompts: [] });
  if (m.method.startsWith("notifications/")) return;
  fail(-32601, "Method not found: " + m.method);
}

let buf = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", d => {
  buf += d;
  let i;
  while ((i = buf.indexOf("\n")) >= 0) {
    const line = buf.slice(0, i).trim();
    buf = buf.slice(i + 1);
    if (!line) continue;
    let m;
    try { m = JSON.parse(line); } catch (_) { send({ jsonrpc: "2.0", id: null, error: { code: -32700, message: "Parse error" } }); continue; }
    handle(m).catch(e => process.stderr.write(String(e) + "\n"));
  }
});
process.stdin.on("end", () => process.exit(0));
