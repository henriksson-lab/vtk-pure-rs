/// A color map that maps scalar values in [0, 1] to RGB colors.
///
/// # Examples
///
/// ```
/// use crate::render::ColorMap;
///
/// let cm = ColorMap::viridis();
/// let color = cm.map(0.5);
/// assert!(color[0] >= 0.0 && color[0] <= 1.0);
///
/// // Map a raw value with a range
/// let c = cm.map_value(50.0, 0.0, 100.0);
///
/// // Look up by name
/// let jet = ColorMap::by_name("jet").unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct ColorMap {
    /// Control points: (parameter, [r, g, b]) where parameter is in [0, 1].
    points: Vec<(f64, [f32; 3])>,
}

impl ColorMap {
    /// Create a color map from control points. Points must be sorted by parameter.
    pub fn new(points: Vec<(f64, [f32; 3])>) -> Self {
        assert!(points.len() >= 2, "need at least 2 control points");
        Self { points }
    }

    /// Map a value in [0, 1] to an RGB color by linear interpolation.
    pub fn map(&self, t: f64) -> [f32; 3] {
        let t = t.clamp(0.0, 1.0);

        if t <= self.points[0].0 {
            return self.points[0].1;
        }
        if t >= self.points[self.points.len() - 1].0 {
            return self.points[self.points.len() - 1].1;
        }

        for i in 0..self.points.len() - 1 {
            let (t0, c0) = self.points[i];
            let (t1, c1) = self.points[i + 1];
            if t >= t0 && t <= t1 {
                let frac = if (t1 - t0).abs() > 1e-10 {
                    ((t - t0) / (t1 - t0)) as f32
                } else {
                    0.0
                };
                return [
                    c0[0] + frac * (c1[0] - c0[0]),
                    c0[1] + frac * (c1[1] - c0[1]),
                    c0[2] + frac * (c1[2] - c0[2]),
                ];
            }
        }

        self.points[self.points.len() - 1].1
    }

    /// Map a raw scalar value using the given range [min, max].
    pub fn map_value(&self, value: f64, min: f64, max: f64) -> [f32; 3] {
        let t = if (max - min).abs() > 1e-10 {
            (value - min) / (max - min)
        } else {
            0.5
        };
        self.map(t)
    }

    // --- Built-in color maps ---

    /// Blue → Cyan → Green → Yellow → Red (rainbow / jet).
    pub fn jet() -> Self {
        Self::new(vec![
            (0.0, [0.0, 0.0, 1.0]),
            (0.25, [0.0, 1.0, 1.0]),
            (0.5, [0.0, 1.0, 0.0]),
            (0.75, [1.0, 1.0, 0.0]),
            (1.0, [1.0, 0.0, 0.0]),
        ])
    }

    /// Cool to warm (blue → white → red), perceptually balanced.
    pub fn cool_to_warm() -> Self {
        Self::new(vec![
            (0.0, [0.231, 0.298, 0.753]),
            (0.5, [0.865, 0.865, 0.865]),
            (1.0, [0.706, 0.016, 0.150]),
        ])
    }

    /// Viridis (dark purple → blue → green → yellow).
    pub fn viridis() -> Self {
        Self::new(vec![
            (0.0, [0.267, 0.004, 0.329]),
            (0.25, [0.282, 0.140, 0.458]),
            (0.5, [0.127, 0.566, 0.551]),
            (0.75, [0.544, 0.774, 0.247]),
            (1.0, [0.993, 0.906, 0.144]),
        ])
    }

    /// Grayscale (black → white).
    pub fn grayscale() -> Self {
        Self::new(vec![(0.0, [0.0, 0.0, 0.0]), (1.0, [1.0, 1.0, 1.0])])
    }

    /// Plasma (dark purple → magenta → orange → yellow).
    pub fn plasma() -> Self {
        Self::new(vec![
            (0.0, [0.050, 0.030, 0.528]),
            (0.25, [0.494, 0.012, 0.658]),
            (0.5, [0.798, 0.280, 0.470]),
            (0.75, [0.973, 0.585, 0.254]),
            (1.0, [0.940, 0.975, 0.131]),
        ])
    }

    /// Inferno (black → dark purple → red → orange → yellow).
    pub fn inferno() -> Self {
        Self::new(vec![
            (0.0, [0.001, 0.000, 0.014]),
            (0.25, [0.341, 0.062, 0.429]),
            (0.5, [0.735, 0.216, 0.330]),
            (0.75, [0.978, 0.557, 0.035]),
            (1.0, [0.988, 0.998, 0.645]),
        ])
    }

    /// Turbo (improved rainbow: blue → cyan → green → yellow → red).
    pub fn turbo() -> Self {
        Self::new(vec![
            (0.0, [0.190, 0.072, 0.232]),
            (0.15, [0.100, 0.380, 0.850]),
            (0.30, [0.120, 0.690, 0.720]),
            (0.45, [0.350, 0.880, 0.380]),
            (0.60, [0.680, 0.930, 0.210]),
            (0.75, [0.950, 0.760, 0.120]),
            (0.90, [0.980, 0.380, 0.060]),
            (1.0, [0.640, 0.120, 0.020]),
        ])
    }

    /// Black-body radiation (black → red → orange → yellow → white).
    pub fn black_body() -> Self {
        Self::new(vec![
            (0.0, [0.0, 0.0, 0.0]),
            (0.33, [0.9, 0.0, 0.0]),
            (0.66, [0.9, 0.9, 0.0]),
            (1.0, [1.0, 1.0, 1.0]),
        ])
    }

    /// Blue-to-red diverging (symmetric around white).
    pub fn blue_red() -> Self {
        Self::new(vec![
            (0.0, [0.0, 0.0, 1.0]),
            (0.5, [1.0, 1.0, 1.0]),
            (1.0, [1.0, 0.0, 0.0]),
        ])
    }

    /// Magma (black → dark purple → red → orange → yellow).
    pub fn magma() -> Self {
        Self::new(vec![
            (0.0, [0.001, 0.000, 0.014]),
            (0.25, [0.270, 0.058, 0.393]),
            (0.5, [0.716, 0.215, 0.475]),
            (0.75, [0.987, 0.535, 0.382]),
            (1.0, [0.987, 0.991, 0.750]),
        ])
    }

    /// Cividis (blue-gray → yellow, color-blind-friendly).
    pub fn cividis() -> Self {
        Self::new(vec![
            (0.0, [0.0, 0.135, 0.305]),
            (0.25, [0.278, 0.290, 0.400]),
            (0.5, [0.478, 0.478, 0.418]),
            (0.75, [0.720, 0.680, 0.320]),
            (1.0, [0.995, 0.905, 0.145]),
        ])
    }

    /// Red-Yellow-Blue diverging.
    pub fn rd_yl_bu() -> Self {
        Self::new(vec![
            (0.0, [0.647, 0.0, 0.149]),
            (0.25, [0.992, 0.553, 0.235]),
            (0.5, [1.0, 1.0, 0.749]),
            (0.75, [0.553, 0.773, 0.871]),
            (1.0, [0.192, 0.212, 0.584]),
        ])
    }

    /// Desaturated rainbow (more perceptually uniform than jet).
    pub fn rainbow_desaturated() -> Self {
        Self::new(vec![
            (0.0, [0.278, 0.278, 0.859]),
            (0.143, [0.0, 0.0, 0.902]),
            (0.285, [0.0, 0.710, 0.902]),
            (0.429, [0.0, 0.804, 0.361]),
            (0.571, [0.478, 0.820, 0.0]),
            (0.714, [0.902, 0.710, 0.0]),
            (0.857, [0.922, 0.384, 0.0]),
            (1.0, [0.847, 0.129, 0.129]),
        ])
    }

    /// Parula (MATLAB-style blue → teal → yellow).
    pub fn parula() -> Self {
        Self::new(vec![
            (0.0, [0.208, 0.166, 0.529]),
            (0.25, [0.078, 0.404, 0.741]),
            (0.5, [0.078, 0.659, 0.604]),
            (0.75, [0.682, 0.741, 0.267]),
            (1.0, [0.976, 0.914, 0.137]),
        ])
    }

    /// Spectral diverging (red → orange → yellow → green → blue).
    pub fn spectral() -> Self {
        Self::new(vec![
            (0.0, [0.620, 0.004, 0.259]),
            (0.25, [0.957, 0.427, 0.263]),
            (0.5, [1.0, 1.0, 0.749]),
            (0.75, [0.533, 0.769, 0.506]),
            (1.0, [0.369, 0.310, 0.635]),
        ])
    }

    /// Get a color map by name.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "jet" => Some(Self::jet()),
            "cool_to_warm" | "coolwarm" => Some(Self::cool_to_warm()),
            "viridis" => Some(Self::viridis()),
            "grayscale" | "gray" => Some(Self::grayscale()),
            "plasma" => Some(Self::plasma()),
            "inferno" => Some(Self::inferno()),
            "turbo" => Some(Self::turbo()),
            "black_body" => Some(Self::black_body()),
            "blue_red" => Some(Self::blue_red()),
            "magma" => Some(Self::magma()),
            "cividis" => Some(Self::cividis()),
            "rd_yl_bu" | "RdYlBu" => Some(Self::rd_yl_bu()),
            "rainbow_desaturated" => Some(Self::rainbow_desaturated()),
            "parula" => Some(Self::parula()),
            "spectral" => Some(Self::spectral()),
            _ => None,
        }
    }

    /// List all built-in color map names.
    pub fn available_names() -> &'static [&'static str] {
        &[
            "jet",
            "cool_to_warm",
            "viridis",
            "grayscale",
            "plasma",
            "inferno",
            "turbo",
            "black_body",
            "blue_red",
            "magma",
            "cividis",
            "rd_yl_bu",
            "rainbow_desaturated",
            "parula",
            "spectral",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jet_endpoints() {
        let cm = ColorMap::jet();
        let c0 = cm.map(0.0);
        assert_eq!(c0, [0.0, 0.0, 1.0]); // blue
        let c1 = cm.map(1.0);
        assert_eq!(c1, [1.0, 0.0, 0.0]); // red
    }

    #[test]
    fn interpolation_midpoint() {
        let cm = ColorMap::grayscale();
        let c = cm.map(0.5);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn map_value_range() {
        let cm = ColorMap::grayscale();
        let c = cm.map_value(50.0, 0.0, 100.0);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn clamp_out_of_range() {
        let cm = ColorMap::jet();
        let below = cm.map(-1.0);
        let above = cm.map(2.0);
        assert_eq!(below, [0.0, 0.0, 1.0]);
        assert_eq!(above, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn all_colormaps_valid() {
        for name in ColorMap::available_names() {
            let cm = ColorMap::by_name(name).unwrap();
            let c = cm.map(0.5);
            assert!(c[0] >= 0.0 && c[0] <= 1.0, "{name}: r out of range");
            assert!(c[1] >= 0.0 && c[1] <= 1.0, "{name}: g out of range");
            assert!(c[2] >= 0.0 && c[2] <= 1.0, "{name}: b out of range");
        }
    }

    #[test]
    fn by_name_lookup() {
        assert!(ColorMap::by_name("viridis").is_some());
        assert!(ColorMap::by_name("magma").is_some());
        assert!(ColorMap::by_name("spectral").is_some());
        assert!(ColorMap::by_name("nonexistent").is_none());
    }

    #[test]
    fn available_names_count() {
        assert_eq!(ColorMap::available_names().len(), 15);
    }
}
