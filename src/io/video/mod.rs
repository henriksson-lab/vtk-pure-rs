#![allow(unexpected_cfgs)]
#[cfg(feature = "ffmpeg")]
mod ffmpeg_writer;
mod frame;
mod ppm_writer;
#[cfg(feature = "ffmpeg")]
pub use ffmpeg_writer::write_video;
pub use frame::FrameSequence;
pub use ppm_writer::write_ppm_sequence;
