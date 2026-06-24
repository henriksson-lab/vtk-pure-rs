//! Depth-of-field post-processing configuration.
//!
//! Simulates camera depth-of-field by blurring regions that are out of focus
//! based on their distance from a focal plane.

/// Depth-of-field configuration.
///
/// Controls the focal distance, aperture (blur strength), and maximum blur
/// radius for the post-processing effect.
#[derive(Debug, Clone)]
pub struct DofConfig {
    /// Whether depth-of-field is enabled.
    pub enabled: bool,
    /// Distance from the camera to the focal plane. Default: 10.0
    pub focal_distance: f32,
    /// Aperture size controlling blur intensity. Default: 0.05
    pub aperture: f32,
    /// Maximum blur radius in pixels. Default: 10.0
    pub max_blur: f32,
}

impl Default for DofConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            focal_distance: 10.0,
            aperture: 0.05,
            max_blur: 10.0,
        }
    }
}

impl DofConfig {
    /// Create enabled DoF with default settings.
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Set focal distance.
    pub fn with_focal_distance(mut self, dist: f32) -> Self {
        self.focal_distance = dist;
        self
    }

    /// Set aperture.
    pub fn with_aperture(mut self, aperture: f32) -> Self {
        self.aperture = aperture;
        self
    }

    /// Set maximum blur radius.
    pub fn with_max_blur(mut self, max_blur: f32) -> Self {
        self.max_blur = max_blur;
        self
    }

    /// Compute the circle of confusion diameter for a given depth.
    ///
    /// CoC = aperture * |depth - focal_distance| / depth
    /// The result is clamped to [0, max_blur].
    pub fn circle_of_confusion(&self, depth: f32) -> f32 {
        if depth <= 0.0 {
            return 0.0;
        }
        let coc = self.aperture * (depth - self.focal_distance).abs() / depth;
        coc.min(self.max_blur)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        let c = DofConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.focal_distance, 10.0);
    }

    #[test]
    fn coc_at_focal_distance_is_zero() {
        let c = DofConfig::new().with_focal_distance(5.0).with_aperture(0.1);
        let coc = c.circle_of_confusion(5.0);
        assert!(
            coc.abs() < 1e-6,
            "CoC at focal distance should be zero, got {}",
            coc
        );
    }
}
