# Web pieces for the "Get the app" (Step 1) QR

The host's **Step 1** QR points at `https://opensource.unisim.co.uk/screens`. With
these two files deployed, scanning it **opens the app if installed** (Android App
Links / iOS Universal Links) or **lands on the download page** if not.

## Files

- `screens/index.html` (+ `screens/icon.png`) → deploy at
  **`https://opensource.unisim.co.uk/screens/`**. A platform-aware landing page
  (Play Store / App Store / PC host download).
- `.well-known/assetlinks.json` → deploy at the **domain root**:
  **`https://opensource.unisim.co.uk/.well-known/assetlinks.json`**
  (served as `application/json`, HTTP 200, **no redirects**). This is what makes
  Android open the app instead of the browser.

## Before it works

1. **Fill in the real URLs** in `screens/index.html` (`PLAY_URL`, `APPSTORE_URL`,
   `HOST_DOWNLOAD_URL`).
2. **Add the production signing fingerprint** to `assetlinks.json`. The one there
   now is the **debug** cert (`CE:14:…`), which only verifies debug builds. For the
   published app, add the **Play App Signing** SHA-256 (Play Console → *Test and
   release → App integrity → App signing key certificate*). App Links accept
   multiple fingerprints, so keep both:
   ```json
   "sha256_cert_fingerprints": ["<debug …>", "<play release …>"]
   ```
3. **iOS Universal Links** need the equivalent `apple-app-site-association` file at
   the domain root once the iOS app exists (Team ID + bundle id) — not included
   yet (iOS is still a scaffold).

## Verify (Android)

```bash
adb shell pm verify-app-links --re-verify com.universalsim.extender
adb shell pm get-app-links com.universalsim.extender   # look for "verified"
```

The app's intent filters are already in `apps/android/.../AndroidManifest.xml`
(`autoVerify="true"` on the `https opensource.unisim.co.uk /screens` link, plus the
`unisimscreens://` custom scheme).
