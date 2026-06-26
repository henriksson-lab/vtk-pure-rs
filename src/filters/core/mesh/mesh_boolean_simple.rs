//! Simple mesh boolean operations using point classification.

use crate::data::{CellArray, Points, PolyData};

/// Classify each vertex of mesh A as inside or outside mesh B using ray casting.
pub fn classify_points(mesh: &PolyData, reference: &PolyData) -> Vec<bool> {
    let n = mesh.points.len();
    (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            point_inside_mesh(p, reference)
        })
        .collect()
}

/// Extract faces of mesh A that are inside mesh B.
pub fn extract_inside(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    extract_classified(mesh_a, mesh_b, true)
}

/// Extract faces of mesh A that are outside mesh B.
pub fn extract_outside(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    extract_classified(mesh_a, mesh_b, false)
}

fn extract_classified(mesh: &PolyData, reference: &PolyData, want_inside: bool) -> PolyData {
    let inside = classify_points(mesh, reference);
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();

    for cell in mesh.polys.iter() {
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let all_match = cell.iter().all(|&v| inside[v as usize] == want_inside);
        if all_match {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }

    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn point_inside_mesh(p: [f64; 3], mesh: &PolyData) -> bool {
    // Ray casting along +X with small jitter to avoid edge hits
    let p = [p[0] + 1e-7, p[1] + 1.3e-7, p[2] + 0.9e-7];
    let mut crossings = 0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if ray_triangle_intersect_x(p, a, b, c) {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

fn ray_triangle_intersect_x(origin: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> bool {
    let dir = [1.0, 0.0, 0.0];
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if det.abs() < 1e-12 {
        return false;
    }
    let inv = 1.0 / det;
    let s = [origin[0] - a[0], origin[1] - a[1], origin[2] - a[2]];
    let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = inv * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > 1e-12
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::core::sources::cube::{cube, CubeParams};
    use crate::filters::core::triangulate::triangulate;
    #[test]
    fn test_inside() {
        // Simple test: a point clearly inside a box
        let big = triangulate(&cube(&CubeParams {
            x_length: 10.0,
            y_length: 10.0,
            z_length: 10.0,
            ..Default::default()
        }));
        let inside = point_inside_mesh([0.0, 0.0, 0.0], &big);
        assert!(inside, "origin should be inside 10x10x10 cube");
    }
    #[test]
    fn test_outside() {
        let big = triangulate(&cube(&CubeParams {
            x_length: 10.0,
            y_length: 10.0,
            z_length: 10.0,
            ..Default::default()
        }));
        let outside = point_inside_mesh([20.0, 0.0, 0.0], &big);
        assert!(!outside, "far point should be outside");
    }
}
