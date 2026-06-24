//! Triangle aspect ratio computation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute aspect ratio for each triangle (ratio of circumradius to inradius * 2).
/// Equilateral triangle = 1.0, degenerate = high value.
pub fn compute_aspect_ratios(mesh: &PolyData) -> PolyData {
    let mut ratios = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            ratios.push(0.0);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        ratios.push(triangle_aspect_ratio(a, b, c));
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AspectRatio",
            ratios,
            1,
        )));
    result
}

/// Get aspect ratio statistics.
pub fn aspect_ratio_stats(mesh: &PolyData) -> (f64, f64, f64) {
    let r = compute_aspect_ratios(mesh);
    let arr = r.cell_data().get_array("AspectRatio").unwrap();
    let n = arr.num_tuples();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mut buf = [0.0f64];
    let mut mn = f64::INFINITY;
    let mut mx = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        mn = mn.min(buf[0]);
        mx = mx.max(buf[0]);
        sum += buf[0];
    }
    (mn, mx, sum / n as f64)
}

fn triangle_aspect_ratio(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ab = edge_len(a, b);
    let bc = edge_len(b, c);
    let ca = edge_len(c, a);
    let s = (ab + bc + ca) * 0.5;
    let area_sq = s * (s - ab) * (s - bc) * (s - ca);
    if area_sq <= 0.0 {
        return f64::INFINITY;
    }
    let area = area_sq.sqrt();
    let circumradius = ab * bc * ca / (4.0 * area);
    let inradius = area / s;
    if inradius < 1e-15 {
        return f64::INFINITY;
    }
    circumradius / (2.0 * inradius)
}

fn edge_len(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equilateral() {
        let h = 3.0f64.sqrt() / 2.0;
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        );
        let (mn, mx, _) = aspect_ratio_stats(&mesh);
        assert!((mn - 1.0).abs() < 0.01);
        assert!((mx - 1.0).abs() < 0.01);
    }
    #[test]
    fn test_thin() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 0.01, 0.0]],
            vec![[0, 1, 2]],
        );
        let (_, mx, _) = aspect_ratio_stats(&mesh);
        assert!(mx > 10.0); // very thin triangle
    }
}
