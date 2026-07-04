/// Type of light source.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    /// Directional light (infinite distance, parallel rays like the sun).
    Directional,
    /// Point light (emits in all directions from a position).
    Point,
    /// Spot light (emits in a cone from a position toward a direction).
    Spot {
        /// Half-angle of the cone in degrees. Default: 30.0
        cone_angle: f64,
        /// Exponent controlling falloff from cone center. Default: 1.0
        exponent: f64,
    },
    /// Ambient light (uniform illumination from all directions).
    Ambient,
}

/// A light source in the scene.
///
/// Analogous to VTK's `vtkLight`. Supports directional, point, spot, and
/// ambient light types with configurable color and intensity.
#[derive(Debug, Clone)]
pub struct Light {
    /// Light type determines how position/direction are interpreted.
    pub light_type: LightType,
    /// Position of the light (used for Point and Spot lights).
    pub position: [f64; 3],
    /// Direction the light points toward (used for Directional and Spot lights).
    pub direction: [f64; 3],
    /// RGB color of the light. Default: white [1, 1, 1].
    pub color: [f32; 3],
    /// Intensity multiplier. Default: 1.0
    pub intensity: f64,
    /// Whether the light is enabled. Default: true
    pub enabled: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            position: [0.0, 0.0, 1.0],
            direction: [0.0, 0.0, -1.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            enabled: true,
        }
    }
}

impl Light {
    /// Create a directional light pointing in the given direction.
    pub fn directional(direction: [f64; 3], color: [f32; 3], intensity: f64) -> Self {
        Self {
            light_type: LightType::Directional,
            direction,
            color,
            intensity,
            ..Default::default()
        }
    }

    /// Create a point light at the given position.
    pub fn point(position: [f64; 3], color: [f32; 3], intensity: f64) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            color,
            intensity,
            ..Default::default()
        }
    }

    /// Create a spot light at the given position pointing toward a direction.
    pub fn spot(
        position: [f64; 3],
        direction: [f64; 3],
        color: [f32; 3],
        intensity: f64,
        cone_angle: f64,
    ) -> Self {
        Self {
            light_type: LightType::Spot {
                cone_angle,
                exponent: 1.0,
            },
            position,
            direction,
            color,
            intensity,
            ..Default::default()
        }
    }

    /// Create an ambient light with the given color and intensity.
    pub fn ambient(color: [f32; 3], intensity: f64) -> Self {
        Self {
            light_type: LightType::Ambient,
            color,
            intensity,
            ..Default::default()
        }
    }

    /// Create a default headlight (directional, white, pointing along -Z).
    pub fn headlight() -> Self {
        Self::directional([0.0, 0.0, -1.0], [1.0, 1.0, 1.0], 1.0)
    }
}

impl std::fmt::Display for Light {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lt = match self.light_type {
            LightType::Directional => "Directional",
            LightType::Point => "Point",
            LightType::Spot { .. } => "Spot",
            LightType::Ambient => "Ambient",
        };
        write!(f, "Light({}, intensity={:.1})", lt, self.intensity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_light() {
        let l = Light::default();
        assert_eq!(l.light_type, LightType::Directional);
        assert!(l.enabled);
        assert_eq!(l.color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn directional_light() {
        let l = Light::directional([1.0, -1.0, 0.0], [1.0, 0.9, 0.8], 0.8);
        assert_eq!(l.light_type, LightType::Directional);
        assert_eq!(l.intensity, 0.8);
    }

    #[test]
    fn point_light() {
        let l = Light::point([5.0, 5.0, 5.0], [1.0, 0.0, 0.0], 2.0);
        assert_eq!(l.light_type, LightType::Point);
        assert_eq!(l.position, [5.0, 5.0, 5.0]);
    }

    #[test]
    fn spot_light() {
        let l = Light::spot(
            [0.0, 10.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, 1.0, 1.0],
            1.5,
            45.0,
        );
        match l.light_type {
            LightType::Spot { cone_angle, .. } => {
                assert_eq!(cone_angle, 45.0);
            }
            _ => panic!("expected spot light"),
        }
    }

    #[test]
    fn ambient_light() {
        let l = Light::ambient([0.2, 0.2, 0.2], 0.5);
        assert_eq!(l.light_type, LightType::Ambient);
        assert_eq!(l.intensity, 0.5);
    }

    #[test]
    fn display() {
        let l = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0], 0.8);
        let s = format!("{l}");
        assert!(s.contains("Directional"));
        assert!(s.contains("0.8"));
    }
}
