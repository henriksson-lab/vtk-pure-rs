//! Frame sequence container.

/// A sequence of RGBA frames for video encoding.
#[derive(Debug, Clone)]
pub struct FrameSequence {
    pub width: u32,
    pub height: u32,
    /// Each frame is width*height*4 bytes of RGBA data.
    pub frames: Vec<Vec<u8>>,
    /// Frames per second. Default: 30.
    pub fps: u32,
}

impl FrameSequence {
    /// Create a new empty frame sequence.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            frames: Vec::new(),
            fps: 30,
        }
    }

    /// Set the frame rate.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Add an RGBA frame. Panics if the frame size doesn't match.
    pub fn add_frame(&mut self, rgba: Vec<u8>) {
        let expected = (self.width * self.height * 4) as usize;
        assert_eq!(
            rgba.len(),
            expected,
            "frame size mismatch: expected {expected}, got {}",
            rgba.len()
        );
        self.frames.push(rgba);
    }

    /// Number of frames.
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    /// Duration in seconds.
    pub fn duration_secs(&self) -> f64 {
        if self.fps == 0 {
            return 0.0;
        }
        self.frames.len() as f64 / self.fps as f64
    }

    /// Convert an RGBA frame to RGB (strip alpha).
    pub fn frame_as_rgb(&self, index: usize) -> Vec<u8> {
        let rgba = &self.frames[index];
        let mut rgb = Vec::with_capacity((self.width * self.height * 3) as usize);
        for pixel in rgba.chunks_exact(4) {
            rgb.push(pixel[0]);
            rgb.push(pixel[1]);
            rgb.push(pixel[2]);
        }
        rgb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_sequence_basic() {
        let mut seq = FrameSequence::new(2, 2);
        seq.add_frame(vec![255; 16]);
        seq.add_frame(vec![0; 16]);
        assert_eq!(seq.num_frames(), 2);
        assert!((seq.duration_secs() - 2.0 / 30.0).abs() < 1e-10);
    }

    #[test]
    fn frame_to_rgb() {
        let mut seq = FrameSequence::new(1, 1);
        seq.add_frame(vec![100, 150, 200, 255]);
        let rgb = seq.frame_as_rgb(0);
        assert_eq!(rgb, vec![100, 150, 200]);
    }

    #[test]
    #[should_panic(expected = "frame size mismatch")]
    fn wrong_frame_size() {
        let mut seq = FrameSequence::new(2, 2);
        seq.add_frame(vec![0; 8]); // should be 16
    }
}
