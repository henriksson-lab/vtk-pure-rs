/// Stereo rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StereoMode {
    /// No stereo (normal mono rendering).
    Off,
    /// Side-by-side (left eye on left half, right eye on right half).
    SideBySide,
    /// Anaglyph red/cyan.
    AnaglyphRedCyan,
    /// Top/bottom (left eye on top, right eye on bottom).
    TopBottom,
}

/// Stereo rendering configuration.
#[derive(Debug, Clone)]
pub struct StereoConfig {
    /// Stereo mode.
    pub mode: StereoMode,
    /// Inter-pupillary distance (eye separation). Default: 0.065 (meters)
    pub eye_separation: f64,
    /// Convergence distance (distance to the zero-parallax plane). Default: 1.0
    pub convergence: f64,
}

impl Default for StereoConfig {
    fn default() -> Self {
        Self {
            mode: StereoMode::Off,
            eye_separation: 0.065,
            convergence: 1.0,
        }
    }
}

impl StereoConfig {
    pub fn side_by_side() -> Self {
        Self {
            mode: StereoMode::SideBySide,
            ..Default::default()
        }
    }

    pub fn anaglyph() -> Self {
        Self {
            mode: StereoMode::AnaglyphRedCyan,
            ..Default::default()
        }
    }

    /// Compute left and right eye camera positions given the main camera.
    pub fn eye_positions(&self, camera_pos: [f64; 3], right: [f64; 3]) -> ([f64; 3], [f64; 3]) {
        let half = self.eye_separation / 2.0;
        let left = [
            camera_pos[0] - right[0] * half,
            camera_pos[1] - right[1] * half,
            camera_pos[2] - right[2] * half,
        ];
        let right_pos = [
            camera_pos[0] + right[0] * half,
            camera_pos[1] + right[1] * half,
            camera_pos[2] + right[2] * half,
        ];
        (left, right_pos)
    }

    pub fn is_stereo(&self) -> bool {
        self.mode != StereoMode::Off
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_off() {
        let s = StereoConfig::default();
        assert!(!s.is_stereo());
    }

    #[test]
    fn side_by_side() {
        let s = StereoConfig::side_by_side();
        assert!(s.is_stereo());
        assert_eq!(s.mode, StereoMode::SideBySide);
    }

    #[test]
    fn eye_positions() {
        let s = StereoConfig {
            eye_separation: 2.0,
            ..Default::default()
        };
        let (left, right) = s.eye_positions([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((left[0] - (-1.0)).abs() < 1e-10);
        assert!((right[0] - 1.0).abs() < 1e-10);
    }
}
