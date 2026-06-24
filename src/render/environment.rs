/// Environment mapping configuration for scene backgrounds and reflections.
///
/// This provides environment map options ranging from simple solid colors
/// to gradient skies and cube map textures.
#[derive(Debug, Clone)]
pub enum EnvironmentMap {
    /// No environment map.
    None,
    /// Uniform solid color background.
    SolidColor([f32; 3]),
    /// Three-stop gradient sky (top, horizon, ground).
    GradientSky {
        top: [f32; 3],
        horizon: [f32; 3],
        ground: [f32; 3],
    },
    /// Six cube map face image paths (+X, -X, +Y, -Y, +Z, -Z).
    CubeMapPaths {
        px: String,
        nx: String,
        py: String,
        ny: String,
        pz: String,
        nz: String,
    },
}

impl Default for EnvironmentMap {
    fn default() -> Self {
        EnvironmentMap::None
    }
}

impl EnvironmentMap {
    /// Returns `true` if no environment map is set.
    pub fn is_none(&self) -> bool {
        matches!(self, EnvironmentMap::None)
    }

    /// A neutral gray gradient suitable for studio-style rendering.
    pub fn studio() -> Self {
        EnvironmentMap::GradientSky {
            top: [0.35, 0.35, 0.40],
            horizon: [0.60, 0.60, 0.60],
            ground: [0.25, 0.25, 0.25],
        }
    }

    /// A blue-sky outdoor gradient.
    pub fn outdoor() -> Self {
        EnvironmentMap::GradientSky {
            top: [0.25, 0.50, 0.90],
            horizon: [0.70, 0.85, 1.00],
            ground: [0.30, 0.25, 0.20],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        let env = EnvironmentMap::default();
        assert!(env.is_none());
        assert!(matches!(env, EnvironmentMap::None));
    }

    #[test]
    fn presets() {
        let studio = EnvironmentMap::studio();
        assert!(!studio.is_none());
        match studio {
            EnvironmentMap::GradientSky {
                top,
                horizon,
                ground,
            } => {
                // Studio should have muted gray tones
                assert!(top[0] > 0.0 && top[0] < 1.0);
                assert!(horizon[0] > top[0]); // horizon brighter than top
                assert!(ground[0] < horizon[0]); // ground darker than horizon
            }
            _ => panic!("expected GradientSky"),
        }

        let outdoor = EnvironmentMap::outdoor();
        assert!(!outdoor.is_none());
        match outdoor {
            EnvironmentMap::GradientSky { top, .. } => {
                // Outdoor sky should be blue-dominant
                assert!(top[2] > top[0]);
            }
            _ => panic!("expected GradientSky"),
        }
    }
}
