//! Find closest point on mesh to a query point.

use crate::data::PolyData;

/// Result of closest point query.
pub struct ClosestPointResult {
    pub point: [f64; 3],
    pub distance: f64,
    pub cell_index: usize,
}

/// Find closest point on mesh surface to query point (brute force).
pub fn closest_point_on_mesh(mesh: &PolyData, query: [f64; 3]) -> Option<ClosestPointResult> {
    let mut best: Option<ClosestPointResult> = None;

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let cp = closest_point_triangle(query, a, b, c);
            let d = dist(query, cp);
            if best.as_ref().map_or(true, |b| d < b.distance) {
                best = Some(ClosestPointResult {
                    point: cp,
                    distance: d,
                    cell_index: ci,
                });
            }
        }
    }
    best
}

/// Find closest vertex (not on surface, just vertex).
pub fn closest_vertex(mesh: &PolyData, query: [f64; 3]) -> (usize, f64) {
    let mut best_idx = 0;
    let mut best_d = f64::INFINITY;
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        let d = dist(query, p);
        if d < best_d {
            best_d = d;
            best_idx = i;
        }
    }
    (best_idx, best_d)
}

fn closest_point_triangle(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
    }

    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_on_surface() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = closest_point_on_mesh(&mesh, [1.0, 1.0, 5.0]).unwrap();
        assert!((r.point[2] - 0.0).abs() < 1e-10); // projected onto plane
        assert!((r.distance - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_vertex() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (idx, d) = closest_vertex(&mesh, [0.1, 0.1, 0.0]);
        assert_eq!(idx, 0);
        assert!(d < 0.2);
    }
}
