use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Compute the intersection lines between two triangle meshes.
///
/// For each pair of triangles (one from each mesh) that intersect,
/// computes the line segment of intersection. Returns a PolyData
/// containing line segments representing the intersection curves.
pub fn intersection_poly_data(a: &PolyData, b: &PolyData) -> PolyData {
    let tris_a = collect_triangles(a);
    let tris_b = collect_triangles(b);

    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut point_map: HashMap<PointKey, i64> = HashMap::new();

    // Brute force: test all pairs (works for moderate meshes)
    for ta in &tris_a {
        for tb in &tris_b {
            if let Some((p1, p2)) = triangle_triangle_intersection(ta, tb) {
                if dist2(p1, p2) > 1e-20 {
                    let i0 = insert_unique_point(&mut points, &mut point_map, p1);
                    let i1 = insert_unique_point(&mut points, &mut point_map, p2);
                    if i0 != i1 {
                        lines.push_cell(&[i0, i1]);
                    }
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd
}

type Tri = [[f64; 3]; 3];

fn collect_triangles(pd: &PolyData) -> Vec<Tri> {
    let mut tris = Vec::new();
    for cell in pd.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = pd.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = pd.points.get(cell[i] as usize);
            let v2 = pd.points.get(cell[i + 1] as usize);
            tris.push([v0, v1, v2]);
        }
    }
    tris
}

/// Compute intersection segment between two triangles.
/// Returns Some((p1, p2)) if they intersect, None otherwise.
fn triangle_triangle_intersection(t1: &Tri, t2: &Tri) -> Option<([f64; 3], [f64; 3])> {
    // Use the approach: intersect each edge of t1 with the plane of t2,
    // then check if intersection point is inside t2, and vice versa.
    let mut seg_points = Vec::new();

    // Edges of t1 vs plane of t2
    let n2 = triangle_normal(t2);
    let d2 = dot(n2, t2[0]);
    for i in 0..3 {
        let a = t1[i];
        let b = t1[(i + 1) % 3];
        if let Some(p) = edge_plane_intersection(a, b, n2, d2) {
            if point_in_triangle(p, t2) {
                seg_points.push(p);
            }
        }
    }

    // Edges of t2 vs plane of t1
    let n1 = triangle_normal(t1);
    let d1 = dot(n1, t1[0]);
    for i in 0..3 {
        let a = t2[i];
        let b = t2[(i + 1) % 3];
        if let Some(p) = edge_plane_intersection(a, b, n1, d1) {
            if point_in_triangle(p, t1) {
                seg_points.push(p);
            }
        }
    }

    if seg_points.len() < 2 {
        return None;
    }

    // Find the two most distant points (segment endpoints)
    let mut best_d2 = 0.0;
    let mut best_i = 0;
    let mut best_j = 1;
    for i in 0..seg_points.len() {
        for j in i + 1..seg_points.len() {
            let d = dist2(seg_points[i], seg_points[j]);
            if d > best_d2 {
                best_d2 = d;
                best_i = i;
                best_j = j;
            }
        }
    }

    Some((seg_points[best_i], seg_points[best_j]))
}

fn triangle_normal(t: &Tri) -> [f64; 3] {
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    cross(e1, e2)
}

fn edge_plane_intersection(a: [f64; 3], b: [f64; 3], normal: [f64; 3], d: f64) -> Option<[f64; 3]> {
    let da = dot(normal, a) - d;
    let db = dot(normal, b) - d;

    if da * db > 0.0 {
        return None; // same side
    }

    let denom = da - db;
    if denom.abs() < 1e-15 {
        return None;
    }

    let t = da / denom;
    Some(lerp3(a, b, t))
}

fn point_in_triangle(p: [f64; 3], t: &Tri) -> bool {
    let v0 = sub(t[2], t[0]);
    let v1 = sub(t[1], t[0]);
    let v2 = sub(p, t[0]);

    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d02 = dot(v0, v2);
    let d11 = dot(v1, v1);
    let d12 = dot(v1, v2);

    let inv_denom = d00 * d11 - d01 * d01;
    if inv_denom.abs() < 1e-20 {
        return false;
    }
    let inv_denom = 1.0 / inv_denom;

    let u = (d11 * d02 - d01 * d12) * inv_denom;
    let v = (d00 * d12 - d01 * d02) * inv_denom;

    u >= -1e-10 && v >= -1e-10 && u + v <= 1.0 + 1e-10
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

type PointKey = (i64, i64, i64);

fn insert_unique_point(
    points: &mut Points<f64>,
    point_map: &mut HashMap<PointKey, i64>,
    point: [f64; 3],
) -> i64 {
    let key = point_key(point);
    *point_map.entry(key).or_insert_with(|| {
        let idx = points.len() as i64;
        points.push(point);
        idx
    })
}

fn point_key(point: [f64; 3]) -> PointKey {
    const SCALE: f64 = 1.0e9;
    (
        (point[0] * SCALE).round() as i64,
        (point[1] * SCALE).round() as i64,
        (point[2] * SCALE).round() as i64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_xy_tri(z: f64) -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([-1.0, -1.0, z]);
        pd.points.push([1.0, -1.0, z]);
        pd.points.push([0.0, 1.0, z]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd
    }

    fn make_xz_tri(y: f64) -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([-1.0, y, -1.0]);
        pd.points.push([1.0, y, -1.0]);
        pd.points.push([0.0, y, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd
    }

    #[test]
    fn perpendicular_triangles() {
        let a = make_xy_tri(0.0);
        let b = make_xz_tri(0.0);
        let result = intersection_poly_data(&a, &b);
        // Two perpendicular triangles should intersect in a line segment
        assert!(result.lines.num_cells() >= 1);
    }

    #[test]
    fn parallel_no_intersection() {
        let a = make_xy_tri(0.0);
        let b = make_xy_tri(1.0); // parallel, offset
        let result = intersection_poly_data(&a, &b);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn disjoint_no_intersection() {
        let a = make_xy_tri(0.0);
        let mut b = PolyData::new();
        b.points.push([10.0, 10.0, -1.0]);
        b.points.push([11.0, 10.0, -1.0]);
        b.points.push([10.5, 10.0, 1.0]);
        b.polys.push_cell(&[0, 1, 2]);
        let result = intersection_poly_data(&a, &b);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let a = PolyData::new();
        let b = make_xy_tri(0.0);
        let result = intersection_poly_data(&a, &b);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
