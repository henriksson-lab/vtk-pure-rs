/// Fog configuration for distance-based atmospheric effects.
#[derive(Debug, Clone, Copy)]
pub struct Fog {
    /// Fog color (RGB). Default: matches background.
    pub color: [f32; 3],
    /// Distance at which fog starts. Default: 10.0
    pub near: f64,
    /// Distance at which fog is fully opaque. Default: 100.0
    pub far: f64,
    /// Fog density for exponential mode. Default: 0.02
    pub density: f64,
    /// Fog mode.
    pub mode: FogMode,
    /// Whether fog is enabled.
    pub enabled: bool,
}

/// Fog blending mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogMode {
    /// Linear interpolation between near and far.
    Linear,
    /// Exponential falloff: exp(-density * distance).
    Exponential,
    /// Squared exponential: exp(-(density * distance)^2).
    ExponentialSquared,
}

impl Default for Fog {
    fn default() -> Self {
        Self {
            color: [0.7, 0.7, 0.8],
            near: 10.0,
            far: 100.0,
            density: 0.02,
            mode: FogMode::Linear,
            enabled: false,
        }
    }
}

impl Fog {
    /// Create linear fog with default colors.
    pub fn linear(near: f64, far: f64) -> Self {
        Self {
            near,
            far,
            enabled: true,
            ..Default::default()
        }
    }

    /// Create exponential fog.
    pub fn exponential(density: f64) -> Self {
        Self {
            density,
            mode: FogMode::Exponential,
            enabled: true,
            ..Default::default()
        }
    }

    /// Set fog color.
    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b];
        self
    }

    /// Compute fog factor (0 = no fog, 1 = fully fogged) for a given distance.
    pub fn factor(&self, distance: f64) -> f64 {
        if !self.enabled {
            return 0.0;
        }
        match self.mode {
            FogMode::Linear => ((distance - self.near) / (self.far - self.near)).clamp(0.0, 1.0),
            FogMode::Exponential => 1.0 - (-self.density * distance).exp(),
            FogMode::ExponentialSquared => {
                let d = self.density * distance;
                1.0 - (-d * d).exp()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_fog() {
        let fog = Fog::linear(10.0, 100.0);
        assert!(fog.enabled);
        assert!((fog.factor(10.0)).abs() < 1e-10);
        assert!((fog.factor(55.0) - 0.5).abs() < 1e-10);
        assert!((fog.factor(100.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn exponential_fog() {
        let fog = Fog::exponential(0.1);
        assert!(fog.factor(0.0).abs() < 1e-10);
        assert!(fog.factor(10.0) > 0.0);
        assert!(fog.factor(100.0) > fog.factor(10.0));
    }

    #[test]
    fn disabled_fog() {
        let fog = Fog::default();
        assert!(!fog.enabled);
        assert_eq!(fog.factor(50.0), 0.0);
    }

    #[test]
    fn with_color() {
        let fog = Fog::linear(1.0, 10.0).with_color(1.0, 0.0, 0.0);
        assert_eq!(fog.color, [1.0, 0.0, 0.0]);
    }
}
