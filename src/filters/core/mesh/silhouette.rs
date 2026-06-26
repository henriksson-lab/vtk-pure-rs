use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Extract silhouette edges from a triangle mesh as seen from a given viewpoint.
///
/// An edge is a silhouette edge if its two incident polygon normals face opposite
/// directions with respect to the viewpoint. Feature edges above VTK's default
/// 60 degree feature angle are also emitted.
///
/// Returns a PolyData with line cells representing the silhouette edges.
pub fn extract_silhouette(input: &PolyData, viewpoint: [f64; 3]) -> PolyData {
    if input.polys.num_cells() == 0 {
        return PolyData::default();
    }

    let feature_angle_cos = 60.0_f64.to_radians().cos();
    let mut edges: HashMap<(i64, i64), TwoNormals> = HashMap::new();

    for cell in input.polys.iter() {
        let np = cell.len();
        if np < 3 {
            continue;
        }
        let normal = polygon_normal(input, cell);

        for j in 0..np {
            let p1 = cell[j];
            let p2 = cell[(j + 1) % np];
            let key = if p1 <= p2 { (p1, p2) } else { (p2, p1) };
            let entry = edges.entry(key).or_default();
            if p1 < p2 {
                entry.left = normal;
            } else {
                entry.right = normal;
            }
        }
    }

    let mut out_lines: CellArray = CellArray::new();
    for (&(p1, p2), normals) in &edges {
        let left_norm = norm(normals.left);
        let right_norm = norm(normals.right);
        let winged = left_norm > 0.5 && right_norm > 0.5;
        let center = midpoint(input.points.get(p1 as usize), input.points.get(p2 as usize));
        let view = [
            viewpoint[0] - center[0],
            viewpoint[1] - center[1],
            viewpoint[2] - center[2],
        ];
        let d1 = dot(view, normals.left);
        let d2 = dot(view, normals.right);
        let edge_angle_cos = dot(normals.left, normals.right);

        if (winged && d1 * d2 < 0.0) || (edge_angle_cos < feature_angle_cos) {
            out_lines.push_cell(&[p1, p2]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.lines = out_lines;
    pd
}

#[derive(Clone, Copy, Default)]
struct TwoNormals {
    left: [f64; 3],
    right: [f64; 3],
}

fn polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut normal = [0.0; 3];
    for i in 0..cell.len() {
        let p = input.points.get(cell[i] as usize);
        let q = input.points.get(cell[(i + 1) % cell.len()] as usize);
        normal[0] += (p[1] - q[1]) * (p[2] + q[2]);
        normal[1] += (p[2] - q[2]) * (p[0] + q[0]);
        normal[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len = norm(normal);
    if len > 1e-15 {
        [normal[0] / len, normal[1] / len, normal[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

fn midpoint(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        0.5 * (a[0] + b[0]),
        0.5 * (a[1] + b[1]),
        0.5 * (a[2] + b[2]),
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_double_triangle() -> PolyData {
        // Two triangles sharing an edge (a "bowtie" seen from above).
        // Triangle 0: (0,0,0), (1,0,0), (0.5,1,0) — in XY plane, normal +Z
        // Triangle 1: (1,0,0), (0.5,1,0), (0.5,0.5,-1) — tilted away
        let mut points: Points<f64> = Points::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.5, 1.0, 0.0]);
        points.push([0.5, 0.5, -1.0]);
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        polys.push_cell(&[1, 2, 3]);
        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd
    }

    #[test]
    fn silhouette_from_above() {
        let mesh = make_double_triangle();
        // Viewpoint far above: first triangle front-facing, second back-facing.
        let result = extract_silhouette(&mesh, [0.5, 0.5, 10.0]);
        // Should have silhouette edges. The shared edge (1,2) should be silhouette
        // since triangle 0 faces up and triangle 1 faces away.
        assert!(result.lines.num_cells() > 0, "Expected silhouette edges");
    }

    #[test]
    fn single_triangle_boundary() {
        let mut points: Points<f64> = Points::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.0, 1.0, 0.0]);
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        let mut mesh = PolyData::new();
        mesh.points = points;
        mesh.polys = polys;
        // Viewpoint above: all 3 boundary edges should be silhouette (front-facing).
        let result = extract_silhouette(&mesh, [0.0, 0.0, 10.0]);
        assert_eq!(result.lines.num_cells(), 3, "3 boundary edges");
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::default();
        let result = extract_silhouette(&mesh, [0.0, 0.0, 1.0]);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
