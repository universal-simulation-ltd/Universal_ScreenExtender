//! M1a probe: confirm ScreenCaptureKit capture works on this machine.
//!
//! Captures the main display for ~3 seconds and reports, for the first few
//! frames, the pixel dimensions, pixel format, and whether the frame is
//! IOSurface-backed (the zero-copy path into VideoToolbox). Ends with a
//! frame count and rough FPS.
//!
//! Run: cargo run -p extender-host --example capture_probe
//! Requires Screen Recording permission (System Settings > Privacy & Security).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use screencapturekit::prelude::*;
use screencapturekit::stream::configuration::PixelFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = SCShareableContent::get()?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or("no displays available (is Screen Recording permission granted?)")?;

    println!(
        "capturing display {} ({}x{} points)",
        display.display_id(),
        display.width(),
        display.height()
    );

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(display.width())
        .with_height(display.height())
        .with_fps(60);

    let count = Arc::new(AtomicUsize::new(0));
    let count_handler = count.clone();

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(
        move |sample: CMSampleBuffer, _ty: SCStreamOutputType| {
            let n = count_handler.fetch_add(1, Ordering::Relaxed);
            if n < 5 {
                match sample.image_buffer() {
                    Some(buffer) if buffer.is_backed_by_io_surface() => {
                        if let Some(surface) = buffer.io_surface() {
                            let raw = surface.pixel_format();
                            println!(
                                "frame {n}: {}x{} px  format={} (0x{raw:08X})  iosurface=yes  scale={:?}",
                                surface.width(),
                                surface.height(),
                                PixelFormat::from(raw),
                                sample.scale_factor(),
                            );
                        }
                    }
                    Some(_) => println!("frame {n}: present but NOT IOSurface-backed"),
                    None => println!("frame {n}: no image buffer"),
                }
            }
        },
        SCStreamOutputType::Screen,
    );

    let started = Instant::now();
    stream.start_capture()?;
    std::thread::sleep(Duration::from_secs(3));
    stream.stop_capture()?;
    let elapsed = started.elapsed().as_secs_f64();

    let total = count.load(Ordering::Relaxed);
    println!(
        "captured {total} frames in {elapsed:.2}s (~{:.1} fps)",
        total as f64 / elapsed
    );
    if total == 0 {
        eprintln!(
            "no frames captured — most likely Screen Recording permission is not granted \
             to the terminal running this. Grant it, then re-run."
        );
    }
    Ok(())
}
