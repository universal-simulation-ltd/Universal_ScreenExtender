# M8f — Phone self-capture (cast a phone's screen into a browser)

**Status:** 🚧 Design only (no code). The net-new capability flagged in
[M8-browser-receiver.md](M8-browser-receiver.md). **Prereq:** M8c (RoomSession +
cast flow) ✅, M8d (browser viewer over the room: `RoomTransport` + M7 decode) ✅,
M8e (WebRTC media) 🚧 optional later.

> **One-line:** make the **phone a video source** — capture its own screen, encode
> H.264, and stream it into a browser receiver — so you can mirror your phone onto
> any screen with a browser (a TV, a projector, a meeting-room display). Today
> phones are client-only (no self-capture); this is the new bit.

## Why

M8c/M8d gave us a browser **viewer** (`RoomTransport` + the M7 decode pipeline) and
a phone that can **drive** a screen. M8f closes the loop the other way: the phone
*is the screen being shown*. The viewer already exists — M8f only adds the phone's
capture + encode + send. So a phone casting to a browser **reuses the entire M8d
browser viewer**: phone = sender (like the desktop host in M8d), browser = receiver.

## The design

The phone produces the **same `postcard` `StreamStart` + `Frame` messages** the
desktop host produces, and sends them into the rendezvous room as the sender. The
browser viewer can't tell the source apart from a desktop host — that's the point.

### Android — MediaProjection → MediaCodec → frames

1. **Consent + capture.** `MediaProjectionManager.createScreenCaptureIntent()` →
   the system consent dialog → a `MediaProjection`. Android 10+ requires a
   **foreground service** with `foregroundServiceType="mediaProjection"` (Android
   14 tightened this — the service must start from a user action and post a
   notification).
2. **Encode.** Feed the projection into a `VirtualDisplay` whose `Surface` is the
   **input surface of a `MediaCodec`** H.264 encoder (hardware). Pull encoded
   buffers out: the codec-config buffer (`BUFFER_FLAG_CODEC_CONFIG`) carries
   **SPS/PPS** → `StreamStart`; subsequent buffers are `Frame`s (keyframe flag from
   `BUFFER_FLAG_KEY_FRAME`).
3. **Frame the bytes.** The app currently has no `postcard` encoder in Kotlin (it
   encodes `Input` via the Rust FFI). For video we need binary `Message` frames, so
   **extend `crates/mobile-ffi`** with `encode_stream_start(w,h,codec,csd)` and
   `encode_frame(pts, data, keyframe)` (the FFI already links `crates/protocol`, so
   this is zero wire-drift). `RoomSession` gains `sendFrame(bytes)` / `sendStart(…)`
   that ship those bytes as **binary** WS messages into the room.
4. **Send.** Reuse `RoomSession` (M8c) — but as a *video* sender: open the room,
   `hello{mode:"mirror"}`, then stream `StreamStart` + `Frame`s.

### iOS — ReplayKit → VideoToolbox → frames

- **In-app capture** (`RPScreenRecorder.startCapture`) hands the app `CMSampleBuffer`s
  for its **own** content — simplest, no extra process; good for casting app/slide
  content. Encode to H.264 with **VideoToolbox** (`VTCompressionSession`) → the same
  `StreamStart`/`Frame` shapes.
- **Whole-device mirror** needs a **Broadcast Upload Extension** (a separate process
  via the system broadcast picker) — more complex (IPC to the app, memory limits).
  Phase it after in-app capture.

### Transport — reuse M8d, upgrade with M8e

- **Relay first:** frames go through the rendezvous DO (binary, like M8d). Fine for
  modest mobile bitrates; it's the same path the desktop uses today.
- **WebRTC later (M8e):** once M8e lands, the phone offers a WebRTC data/media
  channel so the video goes P2P and off the relay. Until then, relay.

## Architecture

```
   phone (NEW: source)                         Cloudflare              browser receiver (reused)
 ┌──────────────────────────┐   binary frames  ┌──────────────┐  binary  ┌────────────────────────┐
 │ MediaProjection (Android)│   into the room   │ Durable      │  frames  │ RoomTransport (M8d)    │
 │  / ReplayKit (iOS)        │─────────────────►│ Object room  │─────────►│  → M7 WebCodecs decode │
 │ → HW H.264 encode         │                   │ (relay)      │          │  → canvas              │
 │ → StreamStart + Frame     │   (M8e: WebRTC P2P bypasses the relay)      │ (unchanged from M8d)   │
 │   via RoomSession (FFI enc)│                  └──────────────┘          └────────────────────────┘
 └──────────────────────────┘
```

## Sub-increments

- **M8f-a — Android capture + encode.** MediaProjection consent + foreground
  service + `VirtualDisplay`→`MediaCodec` H.264. *Gate:* log SPS/PPS + a keyframe
  locally (no network yet).
- **M8f-b — FFI frame encoders + send.** `crates/mobile-ffi` `encode_stream_start`
  / `encode_frame`; `RoomSession.sendStart/sendFrame` ship them as binary into the
  room. *Gate:* a `RoomTransport`-based Node/browser receiver decodes the phone's
  stream (reusing M8d's viewer test harness).
- **M8f-c — browser viewer.** None — **reuse M8d** (`RoomTransport` + M7 decode).
  Just point the browser viewer at the room and confirm the phone's frames render.
- **M8f-d — iOS in-app (ReplayKit + VideoToolbox).** Same frame shapes from iOS.
- **M8f-e — iOS broadcast extension.** Whole-device mirror (separate process).
- **M8f-f — WebRTC (with M8e).** Move the phone's media to WebRTC.

M8f-b is the gate (phone frames decode in a browser); M8f-c is free (M8d reuse).

## Open questions / risks

1. **Android 14 foreground-service rules.** `mediaProjection` FGS must start from a
   user gesture + a visible notification; validate on 14/15 early.
2. **The FFI encode surface.** Adding `encode_stream_start`/`encode_frame` keeps the
   wire canonical (Rust-owned) — confirm the mobile-ffi build (`cargo-ndk`) picks
   them up and the JNI signatures match.
3. **Encoder tuning.** Bitrate/keyframe-interval/resolution for mobile — too high
   chokes the relay (until M8e). Start conservative (e.g. 720p, ~2.5 Mbps, 2 s GOP).
4. **iOS broadcast-extension complexity.** Separate process, memory caps, app IPC —
   sequence it last; ship in-app capture first.
5. **Battery / thermal.** Continuous capture + encode is heavy; show an obvious
   "casting" state + an easy stop, and stop on background.
6. **Audio.** Out of scope for v1 (screen video only); note as a follow-up.

## Surface (planned)

- `apps/android` — a `ScreenCastService` (foreground, MediaProjection),
  `MediaCodec` encoder, and `RoomSession.sendStart/sendFrame`; a "Cast my screen"
  entry. (M8f-a/b)
- `crates/mobile-ffi` — `encode_stream_start` / `encode_frame` (canonical `postcard`
  via `crates/protocol`). (M8f-b)
- `apps/web` — **no change** (reuse `RoomTransport` + the M7 decode from M8d). (M8f-c)
- `apps/ios` — ReplayKit in-app capture → VideoToolbox; later a broadcast extension.
  (M8f-d/e)
