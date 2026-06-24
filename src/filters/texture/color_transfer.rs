/// A piecewise-linear color transfer function.
///
/// Maps scalar values to RGB colors using control points.
/// Similar to a ColorMap but supports arbitrary control points.
#[derive(Debug, Clone)]
pub struct ColorTransferFunction {
    /// Control points: (scalar_value, r, g, b) sorted by scalar_value.
    points: Vec<(f64, f32, f32, f32)>,
}

impl ColorTransferFunction {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    /// Add a control point at the given scalar value with the given color.
    pub fn add_point(&mut self, value: f64, r: f32, g: f32, b: f32) {
        self.points.push((value, r, g, b));
        self.points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    /// Map a scalar value to an RGB color.
    pub fn map_value(&self, value: f64) -> [f32; 3] {
        if self.points.is_empty() {
            return [1.0, 1.0, 1.0];
        }
        if self.points.len() == 1 {
            let p = &self.points[0];
            return [p.1, p.2, p.3];
        }

        // Clamp to range
        if value <= self.points[0].0 {
            let p = &self.points[0];
            return [p.1, p.2, p.3];
        }
        if value >= self.points.last().unwrap().0 {
            let p = self.points.last().unwrap();
            return [p.1, p.2, p.3];
        }

        // Find interval and interpolate
        for i in 0..self.points.len() - 1 {
            let lo = &self.points[i];
            let hi = &self.points[i + 1];
            if value >= lo.0 && value <= hi.0 {
                let t = if (hi.0 - lo.0).abs() > 1e-20 {
                    ((value - lo.0) / (hi.0 - lo.0)) as f32
                } else {
                    0.0
                };
                return [
                    lo.1 + t * (hi.1 - lo.1),
                    lo.2 + t * (hi.2 - lo.2),
                    lo.3 + t * (hi.3 - lo.3),
                ];
            }
        }

        [1.0, 1.0, 1.0]
    }

    /// Create a blue-to-red diverging colormap.
    pub fn blue_to_red() -> Self {
        let mut ctf = Self::new();
        ctf.add_point(0.0, 0.0, 0.0, 1.0); // blue
        ctf.add_point(0.5, 1.0, 1.0, 1.0); // white
        ctf.add_point(1.0, 1.0, 0.0, 0.0); // red
        ctf
    }

    /// Create a rainbow colormap.
    pub fn rainbow() -> Self {
        let mut ctf = Self::new();
        ctf.add_point(0.0, 1.0, 0.0, 0.0); // red
        ctf.add_point(0.25, 1.0, 1.0, 0.0); // yellow
        ctf.add_point(0.5, 0.0, 1.0, 0.0); // green
        ctf.add_point(0.75, 0.0, 1.0, 1.0); // cyan
        ctf.add_point(1.0, 0.0, 0.0, 1.0); // blue
        ctf
    }

    /// Number of control points.
    pub fn num_points(&self) -> usize {
        self.points.len()
    }
}

impl Default for ColorTransferFunction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation() {
        let mut ctf = ColorTransferFunction::new();
        ctf.add_point(0.0, 0.0, 0.0, 0.0);
        ctf.add_point(1.0, 1.0, 1.0, 1.0);

        let c = ctf.map_value(0.5);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn clamping() {
        let mut ctf = ColorTransferFunction::new();
        ctf.add_point(0.0, 0.0, 0.0, 0.0);
        ctf.add_point(1.0, 1.0, 1.0, 1.0);

        let c_lo = ctf.map_value(-1.0);
        assert!(c_lo[0] < 0.01);
        let c_hi = ctf.map_value(2.0);
        assert!(c_hi[0] > 0.99);
    }

    #[test]
    fn blue_to_red_midpoint() {
        let ctf = ColorTransferFunction::blue_to_red();
        let c = ctf.map_value(0.5);
        // Midpoint should be white
        assert!(c[0] > 0.9);
        assert!(c[1] > 0.9);
        assert!(c[2] > 0.9);
    }

    #[test]
    fn rainbow_endpoints() {
        let ctf = ColorTransferFunction::rainbow();
        let c0 = ctf.map_value(0.0);
        assert!(c0[0] > 0.9); // red
        let c1 = ctf.map_value(1.0);
        assert!(c1[2] > 0.9); // blue
    }
}
