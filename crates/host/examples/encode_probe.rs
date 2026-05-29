//! M1c encode probe: validate hardware H.264 encode via VideoToolbox in
//! isolation. Uses a synthetic IOSurface, so it needs NO capture and NO
//! Screen Recording permission. Confirms the `videotoolbox` crate builds and
//! links under Command Line Tools and actually emits an H.264 bitstream.
//!
//! Run: cargo run -p extender-host --example encode_probe

use apple_cf::iosurface::IOSurface;
use videotoolbox::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let surface = IOSurface::create(1920, 1080, u32::from_be_bytes(*b"BGRA"), 4)
        .ok_or("failed to allocate IOSurface")?;

    let encoder = CompressionSession::builder(1920, 1080, Codec::H264)
        .with_real_time(true)
        .with_average_bit_rate(8_000_000)
        .with_expected_frame_rate(60.0)
        .with_max_keyframe_interval(60)
        .build()?;

    let mut total = 0usize;
    for i in 0..120i64 {
        let frame = encoder.encode(&surface, (i, 60))?;
        total += frame.data.len();
        if i < 3 || i % 30 == 0 {
            println!(
                "frame {i:3}: {:>6} bytes  pts={:?}  flags=0x{:x}",
                frame.data.len(),
                frame.presentation_time,
                frame.info_flags
            );
        }
    }
    println!("encoded 120 frames, {total} bytes total of H.264 (synthetic static surface)");
    Ok(())
}
