# Web pieces for the "Get the app" (Step 1) QR

The host's **Step 1** QR points at `https://opensource.unisim.co.uk/screens`. To make
scanning it **open the app if installed** (Android App Links / iOS Universal Links)
or **fall back to your existing `/screens` marketing page** if not, the only file
this repo needs to carry is the Digital Asset Links file.

## The one required file

- `.well-known/assetlinks.json` → deploy at the **domain root**:
  **`https://opensource.unisim.co.uk/.well-known/assetlinks.json`**
  (served as `application/json`, HTTP 200, **no redirects**). This is what makes
  Android open the app instead of the browser. No dedicated landing page is needed
  — when the app isn't installed, the browser just loads your existing `/screens`
  page (with its download links).

## Before it works

1. **Add the production signing fingerprint** to `assetlinks.json`. The one there
   now is the **debug** cert (`CE:14:…`), which only verifies debug builds. For the
   published app, add the **Play App Signing** SHA-256 (Play Console → *Test and
   release → App integrity → App signing key certificate*). App Links accept
   multiple fingerprints — keep both:
   ```json
   "sha256_cert_fingerprints": ["<debug …>", "<play release …>"]
   ```
2. **iOS Universal Links** need an `apple-app-site-association` file at the domain
   root once the iOS app exists (Team ID + bundle id) — not included yet (iOS is
   still a scaffold).

The app's intent filters already target this URL
(`apps/android/.../AndroidManifest.xml`: `autoVerify="true"` on the
`https opensource.unisim.co.uk /screens` link, plus the `unisimscreens://` scheme).

## Optional: smart-banner snippet for the existing /screens page

App Links already auto-open the app when installed, so this is just polish — an
explicit "Open / download" affordance on the marketing page. Paste into `/screens`
and fill in the store URLs:

```html
<script>
  var PLAY = "https://play.google.com/store/apps/details?id=com.universalsim.extender";
  var APPSTORE = "https://apps.apple.com/app/universal-screens/idXXXXXXXXX";
  var ua = navigator.userAgent || "";
  if (/Android/i.test(ua)) location.href = PLAY;          // or render a button
  else if (/iPhone|iPad|iPod/i.test(ua)) location.href = APPSTORE;
  // desktop: leave the page as-is (host download + QR instructions)
</script>
```

## Verify (Android)

```bash
adb shell pm verify-app-links --re-verify com.universalsim.extender
adb shell pm get-app-links com.universalsim.extender   # look for "verified"
```
