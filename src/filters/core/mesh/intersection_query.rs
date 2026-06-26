//! Mesh intersection queries: closest point on surface, distance field,
//! and point-inside-mesh tests.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Find the closest point on a mesh surface to a query point.
///
/// Returns (closest_point, distance, cell_index).
pub fn closest_point_on_surface(
    mesh: &PolyData,
    query: [f64; 3],
) -> Option<([f64; 3], f64, usize)> {
    let mut best_dist2 = f64::MAX;
    let mut best_point = query;
    let mut best_cell = 0;

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        if cell
            .iter()
            .any(|&id| id < 0 || id as usize >= mesh.points.len())
        {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let (cp, d2) = closest_point_on_triangle(query, a, b, c);
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_point = cp;
                best_cell = ci;
            }
        }
    }

    if best_dist2 < f64::MAX {
        Some((best_point, best_dist2.sqrt(), best_cell))
    } else {
        None
    }
}

/// Compute distance from each point in a probe to the nearest surface point.
pub fn surface_distance_field(mesh: &PolyData, probe: &PolyData) -> PolyData {
    let n = probe.points.len();
    let mut distances = Vec::with_capacity(n);
    for i in 0..n {
        let q = probe.points.get(i);
        let d = closest_point_on_surface(mesh, q)
            .map(|(_, d, _)| d)
            .unwrap_or(f64::MAX);
        distances.push(d);
    }
    let mut result = probe.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SurfaceDistance",
            distances,
            1,
        )));
    result
}

/// Project points onto the nearest surface location.
pub fn project_onto_surface(mesh: &PolyData, probe: &PolyData) -> PolyData {
    let n = probe.points.len();
    let mut new_pts = Points::<f64>::new();
    for i in 0..n {
        let q = probe.points.get(i);
        match closest_point_on_surface(mesh, q) {
            Some((cp, _, _)) => new_pts.push(cp),
            None => new_pts.push(q),
        }
    }
    let mut result = probe.clone();
    result.points = new_pts;
    result
}

fn closest_point_on_triangle(
    p: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> ([f64; 3], f64) {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, dist2(p, a));
    }

    let bp = [p[0] - b[0], p[1] - b[1], p[2] - b[2]];
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, dist2(p, b));
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let pt = [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
        return (pt, dist2(p, pt));
    }

    let cp = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, dist2(p, c));
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let pt = [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
        return (pt, dist2(p, pt));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let pt = [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
        return (pt, dist2(p, pt));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let pt = [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ];
    (pt, dist2(p, pt))
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closest_on_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (cp, d, _) = closest_point_on_surface(&mesh, [1.0, 0.5, 1.0]).unwrap();
        assert!((cp[2] - 0.0).abs() < 0.01); // projected onto z=0
        assert!((d - 1.0).abs() < 0.01);
    }
    #[test]
    fn distance_field() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let probe = PolyData::from_points(vec![[0.3, 0.3, 0.5], [5.0, 5.0, 0.0]]);
        let result = surface_distance_field(&mesh, &probe);
        let arr = result.point_data().get_array("SurfaceDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 0.01);
    }
    #[test]
    fn projection() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let probe = PolyData::from_points(vec![[1.0, 0.5, 3.0]]);
        let result = project_onto_surface(&mesh, &probe);
        assert!((result.points.get(0)[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn closest_on_quad_uses_all_polygon_triangles() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let (cp, d, cell_id) = closest_point_on_surface(&mesh, [0.25, 0.75, 1.0]).unwrap();
        assert_eq!(cell_id, 0);
        assert!((cp[0] - 0.25).abs() < 1e-12, "cp={:?}", cp);
        assert!((cp[1] - 0.75).abs() < 1e-12, "cp={:?}", cp);
        assert!((cp[2] - 0.0).abs() < 1e-12, "cp={:?}", cp);
        assert!((d - 1.0).abs() < 1e-12, "d={}", d);
    }
}
