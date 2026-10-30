"""App Store Connect API yardımcısı (yalnızca CI'da, iPad derlemesi için).
Geçici dağıtım sertifikası + App Store profili oluşturur, sonra siler.
Neden: hesap adındaki 'Ç' Apple'ın imza gereksinimini bozuyor (ITMS-90035); kendimiz,
gereksinimi takım koduna bağlayarak imzalayabilmek için yerel bir kimlik gerekiyor.
Kullanım:
  asc.py cert-create <csr.pem> <out.cer>   -> stdout: sertifika id
  asc.py profile-create <cert_id> <bundle> <out.mobileprovision> -> stdout: profil id
  asc.py cert-delete <id> | profile-delete <id>
Ortam: APPLE_API_KEY, APPLE_API_ISSUER, APPLE_API_KEY_PATH"""
import base64, json, os, sys, time, urllib.request, urllib.error
import jwt

API = "https://api.appstoreconnect.apple.com/v1"

def token():
    key = open(os.environ["APPLE_API_KEY_PATH"]).read()
    now = int(time.time())
    return jwt.encode({"iss": os.environ["APPLE_API_ISSUER"], "iat": now, "exp": now + 900, "aud": "appstoreconnect-v1"},
                      key, algorithm="ES256", headers={"kid": os.environ["APPLE_API_KEY"], "typ": "JWT"})

def call(method, path, body=None):
    req = urllib.request.Request(API + path, method=method, data=json.dumps(body).encode() if body else None,
                                 headers={"Authorization": "Bearer " + token(), "Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req) as r:
            t = r.read()
            return json.loads(t) if t else {}
    except urllib.error.HTTPError as e:
        msg = e.read().decode(errors="replace")
        for l in msg.splitlines()[:6]:
            print("::error::ASC " + method + " " + path + ": " + l[:300], file=sys.stderr)
        sys.exit(1)

cmd = sys.argv[1]
if cmd == "cert-create":
    csr = open(sys.argv[2]).read()
    r = call("POST", "/certificates", {"data": {"type": "certificates", "attributes": {"certificateType": "DISTRIBUTION", "csrContent": csr}}})
    open(sys.argv[3], "wb").write(base64.b64decode(r["data"]["attributes"]["certificateContent"]))
    print(r["data"]["id"])
elif cmd == "profile-create":
    cid, bundle, out = sys.argv[2], sys.argv[3], sys.argv[4]
    b = call("GET", "/bundleIds?filter[identifier]=" + bundle + "&limit=5")
    bid = next(x["id"] for x in b["data"] if x["attributes"]["identifier"] == bundle)
    r = call("POST", "/profiles", {"data": {"type": "profiles",
        "attributes": {"name": "Workshop CI " + str(int(time.time())), "profileType": "IOS_APP_STORE"},
        "relationships": {"bundleId": {"data": {"type": "bundleIds", "id": bid}},
                          "certificates": {"data": [{"type": "certificates", "id": cid}]}}}})
    open(out, "wb").write(base64.b64decode(r["data"]["attributes"]["profileContent"]))
    print(r["data"]["id"])
elif cmd == "cert-delete":
    call("DELETE", "/certificates/" + sys.argv[2])
elif cmd == "profile-delete":
    call("DELETE", "/profiles/" + sys.argv[2])
