//! Branded QR codes for the host window — identical logic to `crates/host-windows/src/qr.rs`.

use eframe::egui;
use image::{imageops, Rgba, RgbaImage};

const LOGO_PNG: &[u8] = include_bytes!("../assets/unisim-icon.png");
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/app-icon.png");

pub fn branded_qr(text: &str) -> Option<egui::ColorImage> {
    branded_qr_with(text, LOGO_PNG)
}

pub fn branded_qr_app(text: &str) -> Option<egui::ColorImage> {
    branded_qr_with(text, APP_ICON_PNG)
}

fn branded_qr_with(text: &str, centre_png: &[u8]) -> Option<egui::ColorImage> {
    let code =
        qrcode::QrCode::with_error_correction_level(text.as_bytes(), qrcode::EcLevel::H).ok()?;
    let modules = code.width();
    let colors = code.to_colors();

    let quiet = 4usize;
    let scale = 8usize;
    let dim = ((modules + quiet * 2) * scale) as u32;

    let mut img = RgbaImage::from_pixel(dim, dim, Rgba([255, 255, 255, 255]));
    for y in 0..modules {
        for x in 0..modules {
            if colors[y * modules + x] != qrcode::Color::Dark {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = ((x + quiet) * scale + dx) as u32;
                    let py = ((y + quiet) * scale + dy) as u32;
                    img.put_pixel(px, py, Rgba([0, 0, 0, 255]));
                }
            }
        }
    }

    overlay_logo(&mut img, dim, centre_png);
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [dim as usize, dim as usize],
        &img.into_raw(),
    ))
}

pub fn logo_image(size: u32) -> Option<egui::ColorImage> {
    decode_square(LOGO_PNG, size)
}

pub fn app_icon_image(size: u32) -> Option<egui::ColorImage> {
    decode_square(APP_ICON_PNG, size)
}

pub fn app_icon_rgba(size: u32) -> Option<Vec<u8>> {
    let logo = image::load_from_memory(APP_ICON_PNG)
        .ok()?
        .resize_exact(size, size, imageops::FilterType::Lanczos3)
        .to_rgba8();
    Some(logo.into_raw())
}

fn decode_square(png: &[u8], size: u32) -> Option<egui::ColorImage> {
    let rgba = image::load_from_memory(png)
        .ok()?
        .resize_exact(size, size, imageops::FilterType::Lanczos3)
        .to_rgba8();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [size as usize, size as usize],
        &rgba.into_raw(),
    ))
}

fn overlay_logo(img: &mut RgbaImage, dim: u32, logo_png: &[u8]) {
    let Ok(logo) = image::load_from_memory(logo_png) else { return };
    let logo_size = (dim as f32 * 0.26) as u32;
    let pad = (dim as f32 * 0.32) as u32;
    let logo = logo
        .resize_exact(logo_size, logo_size, imageops::FilterType::Lanczos3)
        .to_rgba8();

    let pad_x = (dim - pad) / 2;
    let pad_y = (dim - pad) / 2;
    for y in pad_y..pad_y + pad {
        for x in pad_x..pad_x + pad {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    let lx = (dim - logo_size) / 2;
    let ly = (dim - logo_size) / 2;
    for (x, y, px) in logo.enumerate_pixels() {
        let a = f32::from(px.0[3]) / 255.0;
        if a <= 0.0 { continue; }
        let (dstx, dsty) = (lx + x, ly + y);
        let bg = img.get_pixel(dstx, dsty).0;
        let blend = |fg: u8, bg: u8| (f32::from(fg) * a + f32::from(bg) * (1.0 - a)) as u8;
        img.put_pixel(
            dstx,
            dsty,
            Rgba([blend(px.0[0], bg[0]), blend(px.0[1], bg[1]), blend(px.0[2], bg[2]), 255]),
        );
    }
}
