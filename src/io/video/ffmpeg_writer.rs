//! FFmpeg-based video encoder (feature-gated behind `ffmpeg`).
//!
//! Requires `ffmpeg-next` crate and system ffmpeg libraries:
//! ```bash
//! apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
//! ```

use crate::io::video::FrameSequence;
use std::path::Path;

/// Video codec selection.
#[derive(Debug, Clone, Copy)]
pub enum VideoCodec {
    /// H.264 (libx264) — widely supported, good compression.
    H264,
    /// H.265/HEVC (libx265) — better compression, less compatible.
    H265,
    /// VP9 — open format, good for web.
    VP9,
    /// MPEG-4 Part 2 — legacy compatibility.
    Mpeg4,
}

/// Video encoding options.
#[derive(Debug, Clone)]
pub struct VideoOptions {
    pub codec: VideoCodec,
    /// Constant Rate Factor (0=lossless, 23=default, 51=worst). Lower = better quality.
    pub crf: u32,
    /// Pixel format. Default: "yuv420p".
    pub pixel_format: String,
}

impl Default for VideoOptions {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            crf: 23,
            pixel_format: "yuv420p".into(),
        }
    }
}

/// Write a frame sequence to a video file using FFmpeg.
///
/// # Example
///
/// ```no_run
/// use crate::io::video::{FrameSequence, write_video, ffmpeg_writer::VideoOptions};
///
/// let mut seq = FrameSequence::new(640, 480).with_fps(30);
/// // ... add frames ...
/// write_video(&seq, std::path::Path::new("output.mp4"), &VideoOptions::default()).unwrap();
/// ```
pub fn write_video(
    seq: &FrameSequence,
    output_path: &Path,
    options: &VideoOptions,
) -> Result<(), String> {
    use ffmpeg_next as ffmpeg;

    ffmpeg::init().map_err(|e| format!("ffmpeg init: {e}"))?;

    let codec_id = match options.codec {
        VideoCodec::H264 => ffmpeg::codec::Id::H264,
        VideoCodec::H265 => ffmpeg::codec::Id::HEVC,
        VideoCodec::VP9 => ffmpeg::codec::Id::VP9,
        VideoCodec::Mpeg4 => ffmpeg::codec::Id::MPEG4,
    };

    let codec = ffmpeg::encoder::find(codec_id)
        .ok_or_else(|| format!("codec {:?} not found", options.codec))?;

    let mut octx =
        ffmpeg::format::output(&output_path).map_err(|e| format!("create output: {e}"))?;

    let mut stream = octx
        .add_stream(codec)
        .map_err(|e| format!("add stream: {e}"))?;

    let encoder_ctx = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|e| format!("encoder context: {e}"))?;

    let mut encoder = encoder_ctx
        .encoder()
        .video()
        .map_err(|e| format!("video encoder: {e}"))?;

    encoder.set_width(seq.width);
    encoder.set_height(seq.height);
    encoder.set_format(ffmpeg::format::Pixel::YUV420P);
    encoder.set_time_base(ffmpeg::Rational::new(1, seq.fps as i32));

    let mut encoder = encoder
        .open_as(codec)
        .map_err(|e| format!("open encoder: {e}"))?;

    stream.set_parameters(&encoder);

    octx.write_header()
        .map_err(|e| format!("write header: {e}"))?;

    // Create scaler for RGB→YUV conversion
    let mut sws = ffmpeg::software::scaling::Context::get(
        ffmpeg::format::Pixel::RGB24,
        seq.width,
        seq.height,
        ffmpeg::format::Pixel::YUV420P,
        seq.width,
        seq.height,
        ffmpeg::software::scaling::Flags::BILINEAR,
    )
    .map_err(|e| format!("scaler: {e}"))?;

    let mut frame_yuv =
        ffmpeg::frame::Video::new(ffmpeg::format::Pixel::YUV420P, seq.width, seq.height);

    let mut frame_rgb =
        ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGB24, seq.width, seq.height);

    for (i, _) in seq.frames.iter().enumerate() {
        let rgb = seq.frame_as_rgb(i);

        // Copy RGB data into frame
        let stride = frame_rgb.stride(0);
        let data = frame_rgb.data_mut(0);
        for y in 0..seq.height as usize {
            let src_start = y * (seq.width as usize) * 3;
            let dst_start = y * stride;
            let row_bytes = (seq.width as usize) * 3;
            data[dst_start..dst_start + row_bytes]
                .copy_from_slice(&rgb[src_start..src_start + row_bytes]);
        }

        // Convert RGB→YUV
        sws.run(&frame_rgb, &mut frame_yuv)
            .map_err(|e| format!("scale frame {i}: {e}"))?;
        frame_yuv.set_pts(Some(i as i64));

        // Encode frame
        encoder
            .send_frame(&frame_yuv)
            .map_err(|e| format!("send frame {i}: {e}"))?;

        let mut packet = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet
                .write_interleaved(&mut octx)
                .map_err(|e| format!("write packet: {e}"))?;
        }
    }

    // Flush encoder
    encoder.send_eof().map_err(|e| format!("send eof: {e}"))?;
    let mut packet = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut packet).is_ok() {
        packet.set_stream(0);
        packet
            .write_interleaved(&mut octx)
            .map_err(|e| format!("write flush packet: {e}"))?;
    }

    octx.write_trailer()
        .map_err(|e| format!("write trailer: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_options_default() {
        let opts = VideoOptions::default();
        assert_eq!(opts.crf, 23);
        assert!(matches!(opts.codec, VideoCodec::H264));
    }
}
