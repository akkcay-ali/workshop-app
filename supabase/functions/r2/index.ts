// Workshop dosyaları için Cloudflare R2 aracısı.
// Yetki: çağıranın oturumuyla public.r2_can(paths) (üyelik + özel pano sahipliği), Supabase depolamasıyla aynı kural.
// İşlemler: sign (GET/PUT için kısa süreli imzalı adres), del, copy, purge (proje sahibi), migrate (geliştirici, eski dosyaları taşır).
import { AwsClient } from "npm:aws4fetch@1.0.20";
import { createClient } from "npm:@supabase/supabase-js@2.45.4";

const ACC = Deno.env.get("R2_ACCOUNT_ID") ?? "";
const BUCKET = Deno.env.get("R2_BUCKET") ?? "workshop";
const r2 = new AwsClient({
  accessKeyId: Deno.env.get("R2_ACCESS_KEY_ID") ?? "",
  secretAccessKey: Deno.env.get("R2_SECRET_ACCESS_KEY") ?? "",
  service: "s3",
  region: "auto",
});
const BASE = `https://${ACC}.r2.cloudflarestorage.com/${BUCKET}/`;
const enc = (p: string) => p.split("/").map(encodeURIComponent).join("/");
const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};
const out = (b: unknown, s = 200) => new Response(JSON.stringify(b), { status: s, headers: { ...CORS, "content-type": "application/json" } });

async function sign(path: string, method: string, ttl: number, type?: string) {
  const u = new URL(BASE + enc(path));
  u.searchParams.set("X-Amz-Expires", String(ttl));
  const headers: Record<string, string> = {};
  if (type) headers["content-type"] = type;
  const r = await r2.sign(u.toString(), { method, headers, aws: { signQuery: true } });
  return r.url;
}
const del = (p: string) => r2.fetch(BASE + enc(p), { method: "DELETE" });

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return new Response("ok", { headers: CORS });
  if (!ACC) return out({ error: "r2_not_configured" }, 500);
  const auth = req.headers.get("Authorization") ?? "";
  const sb = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_ANON_KEY")!, { global: { headers: { Authorization: auth } } });
  let body: any;
  try { body = await req.json(); } catch { return out({ error: "bad_json" }, 400); }
  const op = body.op;
  const can = async (paths: string[]) => {
    if (!Array.isArray(paths) || !paths.length || paths.length > 200) return false;
    const { data, error } = await sb.rpc("r2_can", { paths });
    return !error && data === true;
  };

  try {
    if (op === "ping") {
      const a = await r2.fetch(BASE + "_ping", { method: "PUT", body: "ok" });
      const b = await r2.fetch(BASE + "_ping");
      const t = await b.text();
      await del("_ping");
      const g = await fetch(await sign("_ping2", "PUT", 60, "text/plain"), { method: "PUT", body: "x", headers: { "content-type": "text/plain" } });
      await del("_ping2");
      return out({ put: a.status, get: b.status, body: t, presigned_put: g.status });
    }
    if (op === "sign") {
      const paths: string[] = body.paths ?? [];
      if (!(await can(paths))) return out({ error: "forbidden" }, 403);
      const put = !!body.put;
      const urls = await Promise.all(paths.map((p) => sign(p, put ? "PUT" : "GET", put ? 900 : 3600, put ? body.type : undefined)));
      return out({ urls });
    }
    if (op === "del") {
      const paths: string[] = body.paths ?? [];
      if (!(await can(paths))) return out({ error: "forbidden" }, 403);
      await Promise.all(paths.map(del));
      return out({ ok: true });
    }
    if (op === "copy") {
      const { from, to } = body;
      if (!(await can([from, to]))) return out({ error: "forbidden" }, 403);
      const r = await r2.fetch(BASE + enc(to), { method: "PUT", headers: { "x-amz-copy-source": `/${BUCKET}/${enc(from)}` } });
      if (r.status === 404) return out({ error: "not_found" }, 404);
      if (!r.ok) return out({ error: "copy_failed", status: r.status }, 502);
      return out({ ok: true });
    }
    if (op === "purge") {
      const pid = String(body.pid ?? "");
      const { data: own } = await sb.from("projects").select("id").eq("id", pid).eq("owner", (await sb.auth.getUser()).data.user?.id ?? "").maybeSingle();
      if (!own) return out({ error: "forbidden" }, 403);
      let token = "", n = 0;
      do {
        const u = new URL(BASE);
        u.searchParams.set("list-type", "2");
        u.searchParams.set("prefix", pid + "/");
        if (token) u.searchParams.set("continuation-token", token);
        const x = await (await r2.fetch(u.toString())).text();
        const keys = [...x.matchAll(/<Key>([^<]+)<\/Key>/g)].map((m) => m[1]);
        await Promise.all(keys.map(del));
        n += keys.length;
        token = (x.match(/<NextContinuationToken>([^<]+)<\/NextContinuationToken>/) ?? [])[1] ?? "";
      } while (token);
      return out({ ok: true, n });
    }
    if (op === "migrate") {
      const { data: dev } = await sb.rpc("is_dev");
      if (dev !== true) return out({ error: "forbidden" }, 403);
      const admin = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!);
      const names: string[] = body.names ?? [];
      const res: Record<string, string> = {};
      for (const n of names.slice(0, 40)) {
        const head = await r2.fetch(BASE + enc(n), { method: "HEAD" });
        if (head.ok) { res[n] = "var"; continue; }
        const { data, error } = await admin.storage.from("assets").download(n);
        if (error || !data) { res[n] = "yok"; continue; }
        const r = await r2.fetch(BASE + enc(n), { method: "PUT", body: data, headers: { "content-type": data.type || "application/octet-stream" } });
        res[n] = r.ok ? "taşındı" : "hata " + r.status;
      }
      return out({ res });
    }
    return out({ error: "bad_op" }, 400);
  } catch (e) {
    return out({ error: String(e) }, 500);
  }
});
