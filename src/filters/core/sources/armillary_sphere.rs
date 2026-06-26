//! Armillary sphere (nested rings representing celestial circles).
use crate::data::{CellArray, Points, PolyData};

pub fn armillary_sphere(radius: f64, na: usize) -> PolyData {
    let na = na.max(24);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Ecliptic ring (tilted 23.5 degrees)
    let tilt = 23.5f64 * std::f64::consts::PI / 180.0;
    let rings: Vec<([f64; 3], f64, f64)> = vec![
        ([0.0, 0.0, 1.0], 0.0, radius),                      // equatorial
        ([tilt.sin(), 0.0, tilt.cos()], 0.0, radius * 0.95), // ecliptic
        ([1.0, 0.0, 0.0], 0.0, radius * 0.9),                // meridian 1
        ([0.0, 1.0, 0.0], 0.0, radius * 0.9),                // meridian 2
        (
            [0.0, 0.0, 1.0],
            23.5 * std::f64::consts::PI / 180.0,
            radius * 0.85,
        ), // tropic of cancer (offset circle)
    ];
    for (axis, offset_angle, r) in &rings {
        let rb = pts.len();
        let axis_len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let axis = [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len];
        let circle_radius = r * offset_angle.cos();
        let center = [
            axis[0] * r * offset_angle.sin(),
            axis[1] * r * offset_angle.sin(),
            axis[2] * r * offset_angle.sin(),
        ];
        // Create ring perpendicular to axis
        // Find two orthogonal vectors to axis
        let up = if axis[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = [
            axis[1] * up[2] - axis[2] * up[1],
            axis[2] * up[0] - axis[0] * up[2],
            axis[0] * up[1] - axis[1] * up[0],
        ];
        let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = [u[0] / ul, u[1] / ul, u[2] / ul];
        let v = [
            axis[1] * u[2] - axis[2] * u[1],
            axis[2] * u[0] - axis[0] * u[2],
            axis[0] * u[1] - axis[1] * u[0],
        ];
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            let x = center[0] + circle_radius * (u[0] * a.cos() + v[0] * a.sin());
            let y = center[1] + circle_radius * (u[1] * a.cos() + v[1] * a.sin());
            let z = center[2] + circle_radius * (u[2] * a.cos() + v[2] * a.sin());
            pts.push([x, y, z]);
        }
        for j in 0..na {
            lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
        }
    }
    // Polar axis
    let n_pole = pts.len();
    pts.push([0.0, 0.0, radius * 1.2]);
    let s_pole = pts.len();
    pts.push([0.0, 0.0, -radius * 1.2]);
    lines.push_cell(&[n_pole as i64, s_pole as i64]);
    // Stand
    let stand = pts.len();
    pts.push([0.0, 0.0, -radius * 1.5]);
    lines.push_cell(&[s_pole as i64, stand as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_armillary() {
        let m = armillary_sphere(3.0, 24);
        assert!(m.points.len() > 100);
        assert!(m.lines.num_cells() > 50);
    }
}
