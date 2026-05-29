//! ExtenderScreen wire protocol: message types and framing shared by host and client.
//!
//! Portable, platform-agnostic — no macOS/Windows dependencies live here so the
//! same protocol code compiles on every host and client.

use std::io::{self, Read, Write};

use serde::{Deserialize, Serialize};

/// Protocol version negotiated during the connection handshake.
pub const PROTOCOL_VERSION: u32 = 1;

/// Video codec used for the encoded frame stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Codec {
    H264,
    Hevc,
}

/// A message on the host -> client stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    /// Sent once at stream start: geometry, codec, and the codec parameter
    /// sets (SPS/PPS for H.264) the client needs to build its decoder.
    StreamStart {
        width: u32,
        height: u32,
        codec: Codec,
        parameter_sets: Vec<Vec<u8>>,
    },
    /// One encoded frame (AVCC: length-prefixed NAL units).
    Frame {
        pts_value: i64,
        pts_timescale: i32,
        keyframe: bool,
        data: Vec<u8>,
    },
}

/// Write a length-prefixed, postcard-encoded message to a stream.
///
/// # Errors
/// Returns an error if encoding fails or the underlying writer errors.
pub fn write_message<W: Write>(w: &mut W, msg: &Message) -> io::Result<()> {
    let body = postcard::to_stdvec(msg).map_err(io::Error::other)?;
    let len = u32::try_from(body.len()).map_err(|_| io::Error::other("message too large"))?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(&body)?;
    Ok(())
}

/// Read a length-prefixed, postcard-encoded message from a stream.
///
/// # Errors
/// Returns an error if the stream ends, the length is invalid, or decoding fails.
pub fn read_message<R: Read>(r: &mut R) -> io::Result<Message> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    postcard::from_bytes(&body).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_roundtrip_over_a_buffer() {
        let msgs = vec![
            Message::StreamStart {
                width: 1920,
                height: 1080,
                codec: Codec::H264,
                parameter_sets: vec![vec![1, 2, 3, 4], vec![5, 6]],
            },
            Message::Frame {
                pts_value: 7,
                pts_timescale: 60,
                keyframe: true,
                data: vec![0, 1, 2, 3, 4, 5],
            },
            Message::Frame {
                pts_value: 8,
                pts_timescale: 60,
                keyframe: false,
                data: vec![9, 9, 9],
            },
        ];

        let mut buf = Vec::new();
        for m in &msgs {
            write_message(&mut buf, m).unwrap();
        }

        let mut cursor = io::Cursor::new(buf);
        for expected in &msgs {
            let got = read_message(&mut cursor).unwrap();
            assert_eq!(&got, expected);
        }
    }
}
