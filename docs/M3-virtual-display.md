# M3 — Virtual display (the real "extend")

**Status:** spike done ✅, building. **Prereq:** M1 (streaming) ✅, M2 (input) ✅.

## Goal

Stop mirroring the main screen. The host **creates a real virtual display**, captures *that*, streams it to the client, and routes the client's input *to it*. This is the milestone that turns the project from "remote view of my main screen" into an actual **second screen** — and it dissolves the M2 single-display feedback problem (input now lands on the virtual screen, not the client window).

## Spike result (de-risked)

`crates/host/examples/vdisplay_probe.rs` + `crates/host/shim/virtual_display.m` proved on **macOS 26.2 / arm64**: a private `CGVirtualDisplay` created from our Rust + ObjC-shim stack registers as a genuine active display (count 2→3, `displayID` in `CGGetActiveDisplayList`, 1920×1080), first try, no SIP/entitlement changes, clean teardown on exit. The reverse-engineered interface (Chromium-derived: `CGVirtualDisplayDescriptor`/`Mode`/`Settings`) works as-is. Private class symbols link straight from CoreGraphics.

## Architecture changes

- **Shim** (`virtual_display.m`): already creates + retains a display and returns its `CGDirectDisplayID`. Keep it process-lifetime (the host process *is* the session).
- **Host**: at startup, create the virtual display → `displayID`; wait for it to register; capture **that** display (match `SCDisplay.display_id() == displayID`) instead of `displays().next()`. Map injected input to the **virtual display's global bounds** (`CGDisplay::new(id).bounds()`), so the cursor lands on the virtual screen. One virtual display for the host's lifetime (not per-client).
- **Client**: unchanged — it renders whatever is streamed and sends input.

## Sub-increments

- **M3a — capture the virtual display.** Host creates + captures the virtual display and streams it; input retargeted to the virtual display's bounds (absolute mapping). *Verify:* client shows the empty virtual desktop; drag a window onto the virtual display (via the macOS Displays arrangement) and see it in the client; mouse moves land on the virtual screen.
- **M3b — usable mouse via pointer lock.** Absolute mapping alone is awkward: injecting a move warps the OS cursor onto the virtual display, so it leaves the client window and `CursorMoved` stops. Fix with **relative mouse mode**: grab the cursor in the client window (`winit` `CursorGrabMode::Locked`), hide it, capture raw deltas (`DeviceEvent::MouseMotion`), send `MouseMove` deltas; host accumulates and moves the virtual cursor. Add a release key (e.g. Esc) to ungrab. This makes keyboard work too (focus an app on the virtual screen via the client, then type — no feedback loop).
- **M3c — modes/HiDPI (optional, can fold into M4).** Resolution as a CLI arg; HiDPI/Retina mode; match the virtual display size to the client window.

## Input mapping detail

`CGEvent` mouse coordinates are global (origin = main display top-left). The virtual display occupies a rect in that space. M3a: `global = bounds.origin + normalized * bounds.size` (bounds in points via `CGDisplay::bounds()`; non-HiDPI 1920×1080 → 1920×1080 points). M3b switches mouse to deltas; buttons/scroll/keys unchanged from M2 (they already target the focused app / pointer location, which is now on the virtual screen).

## Permissions

Same as M2: **Screen Recording** (capture — needed for *any* display incl. virtual) + **Accessibility** (injection). No new grants. (Rebuilt unsigned binaries can lose these — re-toggle if capture/injection silently stops.)

## Open questions / risks

1. **Pointer-lock UX** (M3b): how to release the grab (Esc), and cursor visibility. Standard but needs tuning.
2. **HiDPI**: `hiDPI=1` vs `0` and how SCK reports dimensions — start non-HiDPI in M3a for predictable 1:1 mapping.
3. **Teardown**: process-exit removes the display (spike confirmed). A `Drop`/explicit destroy is a nicety for M4.
4. **Window placement**: macOS decides where the virtual display sits in the arrangement; user may need to position it / move windows onto it. A "make primary" or arrangement helper is M4 polish.

## Surface

`crates/host/shim/virtual_display.m` (have) · `crates/host/src/main.rs` (create+capture virtual display, retarget input) · `crates/host/Cargo.toml`/`build.rs` (have). Client untouched for M3a; M3b adds cursor-grab + `DeviceEvent` capture in the client.
