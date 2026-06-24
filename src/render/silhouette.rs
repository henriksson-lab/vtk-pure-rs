/// Configuration for silhouette / outline rendering.
///
/// Renders visible edges of objects as dark outlines. Uses depth-based edge
/// detection as a post-process overlay.
#[derive(Debug, Clone)]
pub struct SilhouetteConfig {
    /// Outline color. Default: black.
    pub color: [f32; 3],
    /// Outline width in pixels (approximate). Default: 1.5
    pub width: f32,
    /// Depth threshold for edge detection. Default: 0.001
    pub depth_threshold: f32,
    /// Normal threshold for edge detection (cosine angle). Default: 0.5
    pub normal_threshold: f32,
    /// Whether silhouette rendering is enabled.
    pub enabled: bool,
}

impl Default for SilhouetteConfig {
    fn default() -> Self {
        Self {
            color: [0.0, 0.0, 0.0],
            width: 1.5,
            depth_threshold: 0.001,
            normal_threshold: 0.5,
            enabled: false,
        }
    }
}

impl SilhouetteConfig {
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b];
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let c = SilhouetteConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.color, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn builder() {
        let c = SilhouetteConfig::new()
            .with_color(1.0, 0.0, 0.0)
            .with_width(2.0);
        assert!(c.enabled);
        assert_eq!(c.color, [1.0, 0.0, 0.0]);
        assert_eq!(c.width, 2.0);
    }
}
