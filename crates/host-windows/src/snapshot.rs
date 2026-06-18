//! Primary-screen capture → JPEG for the clicker's slide preview. Uses plain GDI
//! (`BitBlt` + `GetDIBits`), which is enough for a still preview: it grabs the
//! primary display only and isn't DPI-aware (the image is downscaled for the
//! preview anyway, so logical-vs-physical pixels don't matter here).

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageBuffer, Rgb};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DrawIconEx, GetCursorInfo, GetIconInfo, GetSystemMetrics, CURSORINFO, CURSOR_SHOWING, DI_NORMAL,
    HICON, ICONINFO, SM_CXSCREEN, SM_CYSCREEN,
};

/// Capture the primary display, downscale so its longest side is at most `max_dim`
/// px, and JPEG-encode it at `quality`. Returns `(width, height, jpeg_bytes)` of
/// the (possibly downscaled) image, or `None` if the capture failed.
pub fn capture_primary_jpeg(max_dim: u32, quality: u8) -> Option<(u32, u32, Vec<u8>)> {
    let (w, h, bgra) = unsafe { grab_primary_bgra()? };

    // Windows DIB 32bpp rows are BGRA; repack into an RGB image buffer.
    let mut rgb: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    for (i, px) in rgb.pixels_mut().enumerate() {
        let o = i * 4;
        *px = Rgb([bgra[o + 2], bgra[o + 1], bgra[o]]);
    }

    // Downscale to keep the preview small over the wire.
    let img = DynamicImage::ImageRgb8(rgb);
    let scaled = if w.max(h) > max_dim {
        img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let rgb = scaled.into_rgb8();
    let (sw, sh) = (rgb.width(), rgb.height());

    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality)
        .encode(rgb.as_raw(), sw, sh, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some((sw, sh, out))
}

/// Capture the primary display into a top-down BGRA byte buffer, returning its
/// pixel dimensions and the bytes (tightly packed, `width*4` per row). Returns
/// `None` if any GDI step fails. Also used by the mirror stream.
pub(crate) unsafe fn grab_primary_bgra() -> Option<(u32, u32, Vec<u8>)> {
    let width = GetSystemMetrics(SM_CXSCREEN);
    let height = GetSystemMetrics(SM_CYSCREEN);
    grab_region_bgra(0, 0, width, height)
}

/// Capture an arbitrary virtual-desktop region (`left`/`top` in virtual-screen
/// coordinates, so a secondary/virtual monitor works) into top-down BGRA. Used by
/// the mirror (primary) and extend (virtual monitor) streams.
pub(crate) unsafe fn grab_region_bgra(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Option<(u32, u32, Vec<u8>)> {
    if width <= 0 || height <= 0 {
        return None;
    }

    let hdc_screen = GetDC(None);
    if hdc_screen.0.is_null() {
        return None;
    }
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
    let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
    let old = SelectObject(hdc_mem, hbm.into());

    // Source origin is the region's virtual-screen coordinate (can be negative or
    // beyond the primary), so this grabs whichever monitor that region covers.
    let blt_ok = BitBlt(hdc_mem, 0, 0, width, height, Some(hdc_screen), left, top, SRCCOPY).is_ok();

    // BitBlt doesn't capture the hardware cursor — draw it in ourselves so the
    // pointer is visible (needed to actually use a mirrored/extended screen).
    draw_cursor(hdc_mem, left, top);

    let mut buf = vec![0u8; (width as usize) * (height as usize) * 4];
    // Negative biHeight requests top-down rows (origin at top-left).
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>()).unwrap(),
            biWidth: width,
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let scanlines = GetDIBits(
        hdc_mem,
        hbm,
        0,
        height as u32,
        Some(buf.as_mut_ptr().cast()),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    // Release GDI resources regardless of success.
    SelectObject(hdc_mem, old);
    let _ = DeleteObject(hbm.into());
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    if !blt_ok || scanlines == 0 {
        return None;
    }
    Some((width as u32, height as u32, buf))
}

/// Composite the current mouse cursor into `hdc` (whose origin maps to the
/// captured region's top-left at virtual-screen `left`/`top`). No-op if hidden.
unsafe fn draw_cursor(hdc: HDC, left: i32, top: i32) {
    let mut ci = CURSORINFO {
        cbSize: std::mem::size_of::<CURSORINFO>() as u32,
        ..Default::default()
    };
    if GetCursorInfo(&mut ci).is_err() || ci.flags != CURSOR_SHOWING || ci.hCursor.0.is_null() {
        return;
    }
    let icon = HICON(ci.hCursor.0);
    // The hotspot (e.g. the arrow tip) sits at the cursor position, so offset the
    // icon's top-left by the hotspot and the region origin.
    let mut info = ICONINFO::default();
    let (mut hx, mut hy) = (0i32, 0i32);
    if GetIconInfo(icon, &mut info).is_ok() {
        hx = info.xHotspot as i32;
        hy = info.yHotspot as i32;
        if !info.hbmColor.0.is_null() {
            let _ = DeleteObject(info.hbmColor.into());
        }
        if !info.hbmMask.0.is_null() {
            let _ = DeleteObject(info.hbmMask.into());
        }
    }
    let x = ci.ptScreenPos.x - left - hx;
    let y = ci.ptScreenPos.y - top - hy;
    let _ = DrawIconEx(hdc, x, y, icon, 0, 0, 0, None, DI_NORMAL);
}
