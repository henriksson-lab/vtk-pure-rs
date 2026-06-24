use crate::data::{CellArray, Points, PolyData};
use std::f64::consts::PI;

/// Parameters for generating a circle polyline.
pub struct CircleParams {
    /// Radius. Default: 1.0
    pub radius: f64,
    /// Number of segments. Default: 64
    pub resolution: usize,
    /// Center. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Normal direction of the circle plane. Default: [0, 0, 1]
    pub normal: [f64; 3],
}

impl Default for CircleParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            resolution: 64,
            center: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }
}

/// Generate a circle as a closed polyline.
pub fn circle(params: &CircleParams) -> PolyData {
    let n = params.resolution.max(3);
    let r = params.radius;

    // Build local frame from normal
    let nlen =
        (params.normal[0].powi(2) + params.normal[1].powi(2) + params.normal[2].powi(2)).sqrt();
    let nz = if nlen > 1e-15 {
        [
            params.normal[0] / nlen,
            params.normal[1] / nlen,
            params.normal[2] / nlen,
        ]
    } else {
        [0.0, 0.0, 1.0]
    };

    // Find a perpendicular vector
    let tmp = if nz[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let nx = normalize(cross(nz, tmp));
    let ny = cross(nz, nx);

    let mut points = Points::new();
    let mut ids: Vec<i64> = Vec::with_capacity(n + 1);

    for i in 0..n {
        let angle = 2.0 * PI * i as f64 / n as f64;
        let c = angle.cos();
        let s = angle.sin();
        let x = params.center[0] + r * (c * nx[0] + s * ny[0]);
        let y = params.center[1] + r * (c * nx[1] + s * ny[1]);
        let z = params.center[2] + r * (c * nx[2] + s * ny[2]);
        let idx = points.len() as i64;
        points.push([x, y, z]);
        ids.push(idx);
    }
    ids.push(0); // close the loop

    let mut lines = CellArray::new();
    lines.push_cell(&ids);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [1.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_circle() {
        let pd = circle(&CircleParams::default());
        assert_eq!(pd.points.len(), 64);
        assert_eq!(pd.lines.num_cells(), 1);
    }

    #[test]
    fn points_on_circle() {
        let pd = circle(&CircleParams {
            radius: 2.0,
            resolution: 8,
            center: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        });
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!((r - 2.0).abs() < 1e-10);
            assert!(p[2].abs() < 1e-10);
        }
    }

    #[test]
    fn custom_normal() {
        let pd = circle(&CircleParams {
            resolution: 4,
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        });
        // Circle in XZ plane: all y should be ~0
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            assert!(p[1].abs() < 1e-10);
        }
    }
}
