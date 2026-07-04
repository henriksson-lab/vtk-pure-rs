use crate::data::PolyData;

/// Check if a triangle mesh has any self-intersections.
///
/// Tests all pairs of non-adjacent triangles for intersection.
/// Returns true if any intersection is found.
pub fn has_self_intersection(input: &PolyData) -> bool {
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3], Vec<usize>)> = input
        .polys
        .iter()
        .filter(|c| valid_triangle_cell(input, c))
        .map(|c| {
            let vids: Vec<usize> = c.iter().map(|&id| id as usize).collect();
            (
                input.points.get(c[0] as usize),
                input.points.get(c[1] as usize),
                input.points.get(c[2] as usize),
                vids,
            )
        })
        .collect();

    let nt = tris.len();
    for i in 0..nt {
        for j in i + 1..nt {
            // Skip adjacent triangles (share a vertex)
            let shared = tris[i].3.iter().any(|v| tris[j].3.contains(v));
            if shared {
                continue;
            }

            if tri_tri_intersect(
                &tris[i].0, &tris[i].1, &tris[i].2, &tris[j].0, &tris[j].1, &tris[j].2,
            ) {
                return true;
            }
        }
    }
    false
}

/// Count the number of self-intersecting triangle pairs.
pub fn count_self_intersections(input: &PolyData) -> usize {
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3], Vec<usize>)> = input
        .polys
        .iter()
        .filter(|c| valid_triangle_cell(input, c))
        .map(|c| {
            let vids: Vec<usize> = c.iter().map(|&id| id as usize).collect();
            (
                input.points.get(c[0] as usize),
                input.points.get(c[1] as usize),
                input.points.get(c[2] as usize),
                vids,
            )
        })
        .collect();

    let nt = tris.len();
    let mut count = 0;
    for i in 0..nt {
        for j in i + 1..nt {
            let shared = tris[i].3.iter().any(|v| tris[j].3.contains(v));
            if shared {
                continue;
            }
            if tri_tri_intersect(
                &tris[i].0, &tris[i].1, &tris[i].2, &tris[j].0, &tris[j].1, &tris[j].2,
            ) {
                count += 1;
            }
        }
    }
    count
}

fn valid_triangle_cell(input: &PolyData, cell: &[i64]) -> bool {
    cell.len() == 3
        && cell
            .iter()
            .all(|&pid| pid >= 0 && (pid as usize) < input.points.len())
}

fn tri_tri_intersect(
    a0: &[f64; 3],
    a1: &[f64; 3],
    a2: &[f64; 3],
    b0: &[f64; 3],
    b1: &[f64; 3],
    b2: &[f64; 3],
) -> bool {
    // Simple: check if any edge of A intersects triangle B and vice versa
    let edges_a = [(a0, a1), (a1, a2), (a2, a0)];
    let edges_b = [(b0, b1), (b1, b2), (b2, b0)];

    for &(p, q) in &edges_a {
        if edge_tri_intersect(p, q, b0, b1, b2) {
            return true;
        }
    }
    for &(p, q) in &edges_b {
        if edge_tri_intersect(p, q, a0, a1, a2) {
            return true;
        }
    }

    coplanar_tri_tri_intersect(a0, a1, a2, b0, b1, b2)
}

fn edge_tri_intersect(
    o: &[f64; 3],
    end: &[f64; 3],
    v0: &[f64; 3],
    v1: &[f64; 3],
    v2: &[f64; 3],
) -> bool {
    let d = [end[0] - o[0], end[1] - o[1], end[2] - o[2]];
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return false;
    }
    let f = 1.0 / a;
    let s = [o[0] - v0[0], o[1] - v0[1], o[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > 1e-6 && t < 1.0 - 1e-6 // strictly inside the edge (not at endpoints)
}

fn coplanar_tri_tri_intersect(
    a0: &[f64; 3],
    a1: &[f64; 3],
    a2: &[f64; 3],
    b0: &[f64; 3],
    b1: &[f64; 3],
    b2: &[f64; 3],
) -> bool {
    let n = cross(sub(a1, a0), sub(a2, a0));
    let n_len2 = dot(n, n);
    if n_len2 <= 1e-24 {
        return false;
    }

    let eps = 1e-10 * n_len2.sqrt();
    if dot(n, sub(b0, a0)).abs() > eps
        || dot(n, sub(b1, a0)).abs() > eps
        || dot(n, sub(b2, a0)).abs() > eps
    {
        return false;
    }

    let axis = dominant_axis(n);
    let a = [
        project_point(a0, axis),
        project_point(a1, axis),
        project_point(a2, axis),
    ];
    let b = [
        project_point(b0, axis),
        project_point(b1, axis),
        project_point(b2, axis),
    ];

    for i in 0..3 {
        for j in 0..3 {
            if segments_intersect_2d(a[i], a[(i + 1) % 3], b[j], b[(j + 1) % 3]) {
                return true;
            }
        }
    }

    point_in_tri_2d(a[0], b) || point_in_tri_2d(b[0], a)
}

fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
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

fn dominant_axis(n: [f64; 3]) -> usize {
    let ax = n[0].abs();
    let ay = n[1].abs();
    let az = n[2].abs();
    if ax >= ay && ax >= az {
        0
    } else if ay >= az {
        1
    } else {
        2
    }
}

fn project_point(p: &[f64; 3], drop_axis: usize) -> [f64; 2] {
    match drop_axis {
        0 => [p[1], p[2]],
        1 => [p[0], p[2]],
        _ => [p[0], p[1]],
    }
}

fn segments_intersect_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    const EPS: f64 = 1e-12;
    let o1 = orient_2d(a, b, c);
    let o2 = orient_2d(a, b, d);
    let o3 = orient_2d(c, d, a);
    let o4 = orient_2d(c, d, b);

    if o1.abs() <= EPS && on_segment_2d(a, b, c) {
        return true;
    }
    if o2.abs() <= EPS && on_segment_2d(a, b, d) {
        return true;
    }
    if o3.abs() <= EPS && on_segment_2d(c, d, a) {
        return true;
    }
    if o4.abs() <= EPS && on_segment_2d(c, d, b) {
        return true;
    }

    (o1 > EPS && o2 < -EPS || o1 < -EPS && o2 > EPS)
        && (o3 > EPS && o4 < -EPS || o3 < -EPS && o4 > EPS)
}

fn orient_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn on_segment_2d(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> bool {
    const EPS: f64 = 1e-12;
    p[0] >= a[0].min(b[0]) - EPS
        && p[0] <= a[0].max(b[0]) + EPS
        && p[1] >= a[1].min(b[1]) - EPS
        && p[1] <= a[1].max(b[1]) + EPS
}

fn point_in_tri_2d(p: [f64; 2], tri: [[f64; 2]; 3]) -> bool {
    const EPS: f64 = 1e-12;
    let o0 = orient_2d(tri[0], tri[1], p);
    let o1 = orient_2d(tri[1], tri[2], p);
    let o2 = orient_2d(tri[2], tri[0], p);
    (o0 >= -EPS && o1 >= -EPS && o2 >= -EPS) || (o0 <= EPS && o1 <= EPS && o2 <= EPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_self_intersection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert!(!has_self_intersection(&pd));
        assert_eq!(count_self_intersections(&pd), 0);
    }

    #[test]
    fn crossing_triangles() {
        let mut pd = PolyData::new();
        // Two triangles that cross
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, -0.5, 1.0]);
        pd.points.push([0.0, -0.5, -1.0]);
        pd.points.push([0.0, 0.5, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        assert!(has_self_intersection(&pd));
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(!has_self_intersection(&pd));
    }

    #[test]
    fn ignores_non_triangle_and_invalid_cells() {
        let mut pd = PolyData::new();
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 1.0]);
        pd.points.push([0.0, -0.5, 1.0]);
        pd.points.push([0.0, -0.5, -1.0]);
        pd.points.push([0.0, 0.5, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.polys.push_cell(&[4, 5, 6]);
        pd.polys.push_cell(&[0, 1, 99]);

        assert!(!has_self_intersection(&pd));
        assert_eq!(count_self_intersections(&pd), 0);
    }

    #[test]
    fn detects_coplanar_overlap() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]);
        pd.points.push([1.5, 0.5, 0.0]);
        pd.points.push([0.5, 1.5, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        assert!(has_self_intersection(&pd));
        assert_eq!(count_self_intersections(&pd), 1);
    }
}
