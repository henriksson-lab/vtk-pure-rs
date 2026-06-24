/// Bloom (glow) post-processing configuration.
///
/// Bloom extracts bright areas of the image, blurs them, and adds
/// them back to create a glow effect around bright surfaces.
#[derive(Debug, Clone)]
pub struct BloomConfig {
    /// Brightness threshold above which pixels contribute to bloom. Default: 0.8
    pub threshold: f64,
    /// Bloom intensity multiplier. Default: 0.3
    pub intensity: f64,
    /// Blur radius in pixels. Default: 4.0
    pub radius: f64,
    /// Whether bloom is enabled.
    pub enabled: bool,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            intensity: 0.3,
            radius: 4.0,
            enabled: false,
        }
    }
}

impl BloomConfig {
    /// Create enabled bloom with default settings.
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Set threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set intensity.
    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = intensity;
        self
    }

    /// Generate 1D Gaussian kernel weights for the blur pass.
    pub fn gaussian_weights(&self, kernel_size: usize) -> Vec<f64> {
        let sigma = self.radius / 2.0;
        let mut weights = Vec::with_capacity(kernel_size);
        let center = (kernel_size / 2) as f64;
        let mut sum = 0.0;
        for i in 0..kernel_size {
            let x = i as f64 - center;
            let w = (-x * x / (2.0 * sigma * sigma)).exp();
            weights.push(w);
            sum += w;
        }
        for w in &mut weights {
            *w /= sum;
        }
        weights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disabled() {
        let b = BloomConfig::default();
        assert!(!b.enabled);
    }

    #[test]
    fn enabled() {
        let b = BloomConfig::new().with_threshold(0.9).with_intensity(0.5);
        assert!(b.enabled);
        assert_eq!(b.threshold, 0.9);
    }

    #[test]
    fn gaussian_weights_sum_to_one() {
        let b = BloomConfig::new();
        let w = b.gaussian_weights(9);
        let sum: f64 = w.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn gaussian_weights_symmetric() {
        let b = BloomConfig::new();
        let w = b.gaussian_weights(9);
        assert!((w[0] - w[8]).abs() < 1e-10);
        assert!((w[1] - w[7]).abs() < 1e-10);
    }
}
