use crate::data::PolyData;

/// Check if a triangle mesh has any self-intersections.
///
/// Tests all pairs of non-adjacent triangles for intersection.
/// Returns true if any intersection is found.
pub fn has_self_intersection(input: &PolyData) -> bool {
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3], Vec<usize>)> = input
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
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
        .filter(|c| c.len() >= 3)
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
    false
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
}
