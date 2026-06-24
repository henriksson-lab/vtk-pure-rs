//! Coordinate axes geometry source with labels and arrowheads.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate coordinate axes as line geometry with RGB coloring.
///
/// X=red, Y=green, Z=blue. Includes optional arrowhead cones.
pub fn coordinate_axes(length: f64, with_arrows: bool, resolution: usize) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut colors = Vec::new();

    let axes = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let rgb = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for (ai, axis) in axes.iter().enumerate() {
        let origin_idx = points.len() as i64;
        points.push([0.0, 0.0, 0.0]);
        colors.extend_from_slice(&rgb[ai]);
        let end = [axis[0] * length, axis[1] * length, axis[2] * length];
        let end_idx = points.len() as i64;
        points.push(end);
        colors.extend_from_slice(&rgb[ai]);
        lines.push_cell(&[origin_idx, end_idx]);

        if with_arrows {
            let tip_len = length * 0.15;
            let tip_radius = length * 0.03;
            let n = resolution.max(4);
            let base = [
                end[0] - axis[0] * tip_len,
                end[1] - axis[1] * tip_len,
                end[2] - axis[2] * tip_len,
            ];

            // Local frame
            let up = if axis[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let u = cross(*axis, up);
            let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
            let u = [u[0] / ul, u[1] / ul, u[2] / ul];
            let v = cross(*axis, u);

            let tip_idx = points.len() as i64;
            points.push(end);
            colors.extend_from_slice(&rgb[ai]);

            let cone_base = points.len();
            for i in 0..n {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
                let px = base[0] + tip_radius * (u[0] * angle.cos() + v[0] * angle.sin());
                let py = base[1] + tip_radius * (u[1] * angle.cos() + v[1] * angle.sin());
                let pz = base[2] + tip_radius * (u[2] * angle.cos() + v[2] * angle.sin());
                points.push([px, py, pz]);
                colors.extend_from_slice(&rgb[ai]);
            }

            for i in 0..n {
                let b0 = (cone_base + i) as i64;
                let b1 = (cone_base + (i + 1) % n) as i64;
                polys.push_cell(&[tip_idx, b0, b1]);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGB", colors, 3)));
    mesh
}

/// Generate a simple triad: three line segments from origin along X, Y, Z.
pub fn simple_triad(length: f64) -> PolyData {
    coordinate_axes(length, false, 0)
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
    fn axes_with_arrows() {
        let a = coordinate_axes(1.0, true, 8);
        assert_eq!(a.lines.num_cells(), 3); // 3 axis lines
        assert!(a.polys.num_cells() > 0); // arrowhead cones
        assert!(a.point_data().get_array("RGB").is_some());
    }
    #[test]
    fn simple() {
        let a = simple_triad(2.0);
        assert_eq!(a.lines.num_cells(), 3);
        assert_eq!(a.polys.num_cells(), 0);
    }
}
