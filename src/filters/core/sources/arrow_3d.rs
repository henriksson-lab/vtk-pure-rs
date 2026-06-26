//! 3D arrow source with configurable shaft/tip proportions.

use crate::data::{CellArray, Points, PolyData};

/// Generate a 3D arrow from `start` to `end` with shaft and cone tip.
pub fn arrow_3d(
    start: [f64; 3],
    end: [f64; 3],
    shaft_radius: f64,
    tip_radius: f64,
    tip_fraction: f64,
    resolution: usize,
) -> PolyData {
    let n = resolution.max(4);
    let shaft_radius = shaft_radius.max(0.0);
    let tip_radius = tip_radius.max(0.0);
    let tip_fraction = tip_fraction.clamp(0.0, 1.0);
    let dir = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let length = (dir[0].powi(2) + dir[1].powi(2) + dir[2].powi(2)).sqrt();
    if length < 1e-15 {
        return PolyData::new();
    }
    let d = [dir[0] / length, dir[1] / length, dir[2] / length];

    // Build local frame
    let up = if d[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = cross(d, up);
    let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    let u = [u[0] / ul, u[1] / ul, u[2] / ul];
    let v = cross(d, u);

    let shaft_end_t = 1.0 - tip_fraction;
    let shaft_end = [
        start[0] + dir[0] * shaft_end_t,
        start[1] + dir[1] * shaft_end_t,
        start[2] + dir[2] * shaft_end_t,
    ];

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Shaft cylinder
    for layer in 0..2 {
        let center = if layer == 0 { start } else { shaft_end };
        for i in 0..=n {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            let px = center[0] + shaft_radius * (u[0] * angle.cos() + v[0] * angle.sin());
            let py = center[1] + shaft_radius * (u[1] * angle.cos() + v[1] * angle.sin());
            let pz = center[2] + shaft_radius * (u[2] * angle.cos() + v[2] * angle.sin());
            points.push([px, py, pz]);
        }
    }
    let row = n + 1;
    for i in 0..n {
        let p0 = i as i64;
        let p1 = p0 + 1;
        let p2 = p1 + row as i64;
        let p3 = p0 + row as i64;
        polys.push_cell(&[p0, p1, p2]);
        polys.push_cell(&[p0, p2, p3]);
    }

    let start_center = points.len() as i64;
    points.push(start);
    for i in 0..n {
        polys.push_cell(&[start_center, (i + 1) as i64, i as i64]);
    }

    let shaft_end_center = points.len() as i64;
    points.push(shaft_end);
    for i in 0..n {
        polys.push_cell(&[shaft_end_center, (row + i) as i64, (row + i + 1) as i64]);
    }

    // Tip cone
    let cone_base_offset = points.len();
    for i in 0..=n {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        let px = shaft_end[0] + tip_radius * (u[0] * angle.cos() + v[0] * angle.sin());
        let py = shaft_end[1] + tip_radius * (u[1] * angle.cos() + v[1] * angle.sin());
        let pz = shaft_end[2] + tip_radius * (u[2] * angle.cos() + v[2] * angle.sin());
        points.push([px, py, pz]);
    }
    let tip_idx = points.len() as i64;
    points.push(end);

    for i in 0..n {
        let b0 = (cone_base_offset + i) as i64;
        polys.push_cell(&[b0, b0 + 1, tip_idx]);
    }

    let cone_base_center = points.len() as i64;
    points.push(shaft_end);
    for i in 0..n {
        let b0 = (cone_base_offset + i) as i64;
        polys.push_cell(&[cone_base_center, b0 + 1, b0]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let a = arrow_3d([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], 0.1, 0.2, 0.3, 8);
        assert!(a.points.len() > 10);
        assert!(a.polys.num_cells() > 10);
    }
    #[test]
    fn x_direction() {
        let a = arrow_3d([0.0, 0.0, 0.0], [5.0, 0.0, 0.0], 0.1, 0.2, 0.2, 6);
        assert!(a.polys.num_cells() > 0);
    }
    #[test]
    fn zero_length() {
        let a = arrow_3d([1.0, 1.0, 1.0], [1.0, 1.0, 1.0], 0.1, 0.2, 0.3, 8);
        assert_eq!(a.points.len(), 0);
    }

    #[test]
    fn clamps_tip_fraction() {
        let a = arrow_3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.1, 0.2, 2.0, 8);
        assert!(a.polys.num_cells() > 0);
    }
}
