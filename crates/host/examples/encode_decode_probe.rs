//! M1c encode->decode roundtrip probe: validate the full VideoToolbox codec
//! roundtrip (H.264 encode then decode) in isolation. Synthetic IOSurface, so
//! NO capture and NO Screen Recording permission needed. Confirms both
//! VTCompressionSession and VTDecompressionSession work under Command Line Tools.
//!
//! Run: cargo run -p extender-host --example encode_decode_probe

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use apple_cf::iosurface::IOSurface;
use videotoolbox::prelude::*;
use videotoolbox::{DecodedFrame, DecompressionSession};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let surface = IOSurface::create(1920, 1080, u32::from_be_bytes(*b"BGRA"), 4)
        .ok_or("failed to allocate IOSurface")?;

    let encoder = CompressionSession::builder(1920, 1080, Codec::H264)
        .with_real_time(true)
        .with_average_bit_rate(8_000_000)
        .with_expected_frame_rate(60.0)
        .with_max_keyframe_interval(60)
        .build()?;

    // Encode the first (keyframe) frame to obtain the stream's format description,
    // which the decoder needs to open a session.
    let first = encoder.encode(&surface, (0, 60))?;
    let sb0 = first
        .cm_sample_buffer()
        .ok_or("encoder produced no sample buffer")?;
    let fmt = sb0
        .format_description()
        .ok_or("first encoded frame has no format description")?;

    let decoded = Arc::new(AtomicUsize::new(0));
    let counter = decoded.clone();
    let decoder = DecompressionSession::new(&fmt, move |frame: DecodedFrame| {
        if let Some(img) = frame.image_buffer {
            let n = counter.fetch_add(1, Ordering::Relaxed);
            if n < 3 {
                println!(
                    "decoded frame {n}: {}x{} pixfmt=0x{:08X} status={}",
                    img.width(),
                    img.height(),
                    img.pixel_format(),
                    frame.status
                );
            }
        } else {
            eprintln!("decoded callback with no image buffer (status={})", frame.status);
        }
    })?;

    decoder.decode(sb0)?;
    for i in 1..30i64 {
        let f = encoder.encode(&surface, (i, 60))?;
        if let Some(sb) = f.cm_sample_buffer() {
            decoder.decode(sb)?;
        }
    }
    decoder.wait_for_async_frames()?;

    println!(
        "roundtrip OK: encoded + decoded {} frames",
        decoded.load(Ordering::Relaxed)
    );
    Ok(())
}
