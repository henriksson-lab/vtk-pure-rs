/// Screen-space ambient occlusion configuration.
#[derive(Debug, Clone)]
pub struct SsaoConfig {
    /// Whether SSAO is enabled.
    pub enabled: bool,
    /// Radius of the sampling hemisphere in world units. Default: 0.5
    pub radius: f32,
    /// AO intensity multiplier. Default: 1.0
    pub intensity: f32,
    /// Bias to prevent self-occlusion artifacts. Default: 0.025
    pub bias: f32,
    /// Number of samples per pixel (max 32). Default: 16
    pub num_samples: u32,
}

impl Default for SsaoConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            radius: 0.5,
            intensity: 1.0,
            bias: 0.025,
            num_samples: 16,
        }
    }
}

impl SsaoConfig {
    /// Create enabled SSAO with default settings.
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn with_samples(mut self, num_samples: u32) -> Self {
        self.num_samples = num_samples.min(32);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        let c = SsaoConfig::default();
        assert!(!c.enabled);
    }

    #[test]
    fn enabled_builder() {
        let c = SsaoConfig::new()
            .with_radius(1.0)
            .with_intensity(0.5)
            .with_samples(32);
        assert!(c.enabled);
        assert_eq!(c.radius, 1.0);
        assert_eq!(c.num_samples, 32);
    }
}
