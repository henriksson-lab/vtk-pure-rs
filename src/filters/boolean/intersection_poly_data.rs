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
    const TOLERANCE: f64 = 1.0e-6;
    const COPLANAR_TOLERANCE: f64 = 1.0e-9;

    let n1 = normalize(triangle_normal(t1))?;
    let n2 = normalize(triangle_normal(t2))?;
    let s1 = -dot(n1, t1[0]);
    let s2 = -dot(n2, t2[0]);

    let dist1 = [
        dot(n2, t1[0]) + s2,
        dot(n2, t1[1]) + s2,
        dot(n2, t1[2]) + s2,
    ];
    if dist1[0] * dist1[1] > TOLERANCE && dist1[0] * dist1[2] > TOLERANCE {
        return None;
    }

    let dist2 = [
        dot(n1, t2[0]) + s1,
        dot(n1, t2[1]) + s1,
        dot(n1, t2[2]) + s1,
    ];
    if dist2[0] * dist2[1] > TOLERANCE && dist2[0] * dist2[2] > TOLERANCE {
        return None;
    }

    if (n1[0] - n2[0]).abs() < COPLANAR_TOLERANCE
        && (n1[1] - n2[1]).abs() < COPLANAR_TOLERANCE
        && (n1[2] - n2[2]).abs() < COPLANAR_TOLERANCE
        && (s1 - s2).abs() < COPLANAR_TOLERANCE
    {
        return None;
    }

    let n1n2 = dot(n1, n2);
    let denom = n1n2 * n1n2 - 1.0;
    if denom.abs() < 1.0e-15 {
        return None;
    }

    let a = (s1 - s2 * n1n2) / denom;
    let b = (s2 - s1 * n1n2) / denom;
    let p = [
        a * n1[0] + b * n2[0],
        a * n1[1] + b * n2[1],
        a * n1[2] + b * n2[2],
    ];
    let v = normalize(cross(n1, n2))?;

    let mut range1 = edge_plane_parameters(t1, n2, t2[0], p, v, TOLERANCE)?;
    let mut range2 = edge_plane_parameters(t2, n1, t1[0], p, v, TOLERANCE)?;
    if range1[0] > range1[1] {
        range1.swap(0, 1);
    }
    if range2[0] > range2[1] {
        range2.swap(0, 1);
    }

    if range1[1] < range2[0] || range2[1] < range1[0] {
        return None;
    }

    let t_start = range1[0].max(range2[0]);
    let t_end = range1[1].min(range2[1]);
    if !t_start.is_finite() || !t_end.is_finite() || (t_end - t_start).abs() <= TOLERANCE {
        return None;
    }

    Some((add_scaled(p, v, t_start), add_scaled(p, v, t_end)))
}

fn triangle_normal(t: &Tri) -> [f64; 3] {
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    cross(e1, e2)
}

fn edge_plane_parameters(
    tri: &Tri,
    normal: [f64; 3],
    plane_point: [f64; 3],
    line_point: [f64; 3],
    line_dir: [f64; 3],
    tolerance: f64,
) -> Option<[f64; 2]> {
    let mut values = Vec::with_capacity(3);
    for i in 0..3 {
        let a = tri[i];
        let b = tri[(i + 1) % 3];
        if let Some((t, x)) = plane_intersect_segment(a, b, normal, plane_point, tolerance) {
            if t >= -tolerance && t <= 1.0 + tolerance {
                values.push(dot(sub(x, line_point), line_dir));
            }
        }
    }

    if values.len() > 2 {
        values.sort_by(|a, b| a.total_cmp(b));
        values.dedup_by(|a, b| (*a - *b).abs() <= tolerance);
    }

    if values.len() != 2 || values.iter().any(|value| value.is_nan()) {
        return None;
    }

    Some([values[0], values[1]])
}

fn plane_intersect_segment(
    a: [f64; 3],
    b: [f64; 3],
    normal: [f64; 3],
    plane_point: [f64; 3],
    tolerance: f64,
) -> Option<(f64, [f64; 3])> {
    let ab = sub(b, a);
    let denom = dot(normal, ab);
    if denom.abs() <= 1.0e-15 {
        return None;
    }

    let t = dot(normal, sub(plane_point, a)) / denom;
    if t < -tolerance || t > 1.0 + tolerance {
        return None;
    }
    Some((t, add_scaled(a, ab, t)))
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

fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let norm = dot(v, v).sqrt();
    if norm <= 1.0e-20 {
        None
    } else {
        Some([v[0] / norm, v[1] / norm, v[2] / norm])
    }
}

fn add_scaled(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + t * b[0], a[1] + t * b[1], a[2] + t * b[2]]
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

    #[test]
    fn shared_vertex_triangles_do_not_intersect() {
        let t1 = [
            [-30.125, 29.3125, -27.1875],
            [-29.9375, 29.375, -27.3125],
            [-30.0625, 28.5, -27.25],
        ];
        let t2 = [
            [-29.9375, 29.3125, -27.3125],
            [-29.875, 29.8125, -27.5],
            [-29.75, 27.6875, -27.4375],
        ];

        assert!(triangle_triangle_intersection(&t1, &t2).is_none());
    }
}
