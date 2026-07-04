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

    /// Compute the VTK depth-of-field shader circle of confusion from depth-buffer z.
    ///
    /// This mirrors `vtkDepthOfFieldPassFS.glsl`:
    /// `CoC = focalDisk * focalDistance * (far - near) / (far * near) * z
    ///      + focalDisk * (near - focalDistance) / near`.
    /// Like the shader, this preserves sign and does not clamp to `max_blur`.
    pub fn circle_of_confusion_depth_buffer(&self, z: f32, near_clip: f32, far_clip: f32) -> f32 {
        if !near_clip.is_finite()
            || !far_clip.is_finite()
            || near_clip <= 0.0
            || far_clip <= near_clip
        {
            return 0.0;
        }

        let coc_scale =
            self.aperture * self.focal_distance * (far_clip - near_clip) / (far_clip * near_clip);
        let coc_bias = self.aperture * (near_clip - self.focal_distance) / near_clip;
        coc_scale * z + coc_bias
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

    #[test]
    fn vtk_depth_buffer_coc_at_focal_depth_is_zero() {
        let c = DofConfig::new()
            .with_focal_distance(5.0)
            .with_aperture(0.1)
            .with_max_blur(10.0);
        let near = 1.0;
        let far = 10.0;
        let z = far * (near - c.focal_distance) / (c.focal_distance * (near - far));
        assert!(c.circle_of_confusion_depth_buffer(z, near, far).abs() < 1e-5);
    }

    #[test]
    fn vtk_depth_buffer_coc_preserves_sign() {
        let c = DofConfig::new()
            .with_focal_distance(5.0)
            .with_aperture(0.1)
            .with_max_blur(10.0);
        assert!(c.circle_of_confusion_depth_buffer(0.0, 1.0, 10.0) < 0.0);
        assert!(c.circle_of_confusion_depth_buffer(1.0, 1.0, 10.0) > 0.0);
    }

    #[test]
    fn vtk_depth_buffer_coc_is_not_clamped_by_max_blur() {
        let c = DofConfig::new()
            .with_focal_distance(100.0)
            .with_aperture(1.0)
            .with_max_blur(0.5);
        assert!(c.circle_of_confusion_depth_buffer(1.0, 1.0, 10.0).abs() > c.max_blur);
    }
}
