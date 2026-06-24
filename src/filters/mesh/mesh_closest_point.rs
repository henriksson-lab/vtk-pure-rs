//! Project a set of query points onto the nearest mesh surface.
use crate::data::{CellArray, Points, PolyData};

pub fn closest_point_projection(mesh: &PolyData, queries: &[[f64; 3]]) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for q in queries {
        let mut best_dist = f64::INFINITY;
        let mut best_pt = *q;
        for &[a, b, c] in &tris {
            if a >= n || b >= n || c >= n {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let pt = closest_point_on_triangle(q, pa, pb, pc);
            let d = (pt[0] - q[0]).powi(2) + (pt[1] - q[1]).powi(2) + (pt[2] - q[2]).powi(2);
            if d < best_dist {
                best_dist = d;
                best_pt = pt;
            }
        }
        let idx = pts.len();
        pts.push(best_pt);
        verts.push_cell(&[idx as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.verts = verts;
    m
}

fn closest_point_on_triangle(p: &[f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let d1 = ab[0] * ap[0] + ab[1] * ap[1] + ab[2] * ap[2];
    let d2 = ac[0] * ap[0] + ac[1] * ap[1] + ac[2] * ap[2];
    if d1 <= 0.0 && d2 <= 0.0 {
        return [a[0], a[1], a[2]];
    }
    let bp = [p[0] - b[0], p[1] - b[1], p[2] - b[2]];
    let d3 = ab[0] * bp[0] + ab[1] * bp[1] + ab[2] * bp[2];
    let d4 = ac[0] * bp[0] + ac[1] * bp[1] + ac[2] * bp[2];
    if d3 >= 0.0 && d4 <= d3 {
        return [b[0], b[1], b[2]];
    }
    let cp = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let d5 = ab[0] * cp[0] + ab[1] * cp[1] + ab[2] * cp[2];
    let d6 = ac[0] * cp[0] + ac[1] * cp[1] + ac[2] * cp[2];
    if d6 >= 0.0 && d5 <= d6 {
        return [c[0], c[1], c[2]];
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_closest() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let queries = vec![[0.5, 0.3, 1.0]]; // above the triangle
        let r = closest_point_projection(&mesh, &queries);
        let p = r.points.get(0);
        assert!((p[2]).abs() < 1e-9); // projected onto z=0 plane
    }
}
