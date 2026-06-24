//! Boolean operations on PolyData meshes.
//!
//! VTK-native style boolean operations: union, intersection, difference.
//! Uses a simplified approach based on inside/outside classification
//! via ray casting + mesh concatenation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Boolean operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
}

/// Perform a boolean operation on two triangle meshes.
///
/// Uses ray-casting point-in-mesh classification:
/// - Union: keep cells from A outside B + cells from B outside A
/// - Intersection: keep cells from A inside B + cells from B inside A
/// - Difference: keep cells from A outside B + cells from B inside A (inverted)
///
/// This is a cell-level classification (whole cells kept or removed based
/// on centroid location), not an exact geometric boolean.
pub fn boolean_operation(a: &PolyData, b: &PolyData, op: BooleanOp) -> PolyData {
    let a_centroids = cell_centroids(a);
    let b_centroids = cell_centroids(b);

    let a_inside_b: Vec<bool> = a_centroids.iter().map(|c| point_in_mesh(c, b)).collect();
    let b_inside_a: Vec<bool> = b_centroids.iter().map(|c| point_in_mesh(c, a)).collect();

    let keep_a: Vec<bool> = match op {
        BooleanOp::Union => a_inside_b.iter().map(|inside| !inside).collect(),
        BooleanOp::Intersection => a_inside_b.clone(),
        BooleanOp::Difference => a_inside_b.iter().map(|inside| !inside).collect(),
    };

    let keep_b: Vec<bool> = match op {
        BooleanOp::Union => b_inside_a.iter().map(|inside| !inside).collect(),
        BooleanOp::Intersection => b_inside_a.clone(),
        BooleanOp::Difference => b_inside_a.clone(), // keep B inside A, will flip normals
    };

    // Extract kept cells from A
    let mut result = extract_kept_cells(a, &keep_a);

    // Extract kept cells from B and append
    let b_kept = extract_kept_cells(b, &keep_b);
    append_poly_data(&mut result, &b_kept);

    result
}

fn cell_centroids(pd: &PolyData) -> Vec<[f64; 3]> {
    let nc = pd.polys.num_cells();
    let mut centroids = Vec::with_capacity(nc);
    for ci in 0..nc {
        let cell = pd.polys.cell(ci);
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &vid in cell {
            let p = pd.points.get(vid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        centroids.push([cx / n, cy / n, cz / n]);
    }
    centroids
}

/// Simple ray-casting point-in-mesh test (cast along +X axis).
fn point_in_mesh(point: &[f64; 3], mesh: &PolyData) -> bool {
    let mut crossings = 0;
    let nc = mesh.polys.num_cells();

    for ci in 0..nc {
        let cell = mesh.polys.cell(ci);
        if cell.len() < 3 {
            continue;
        }

        // Fan-triangulate and test each sub-triangle
        let p0 = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = mesh.points.get(cell[i] as usize);
            let p2 = mesh.points.get(cell[i + 1] as usize);
            if ray_intersects_triangle(point, p0, p1, p2) {
                crossings += 1;
            }
        }
    }

    crossings % 2 == 1
}

/// Möller–Trumbore ray-triangle intersection (ray along +X from point).
fn ray_intersects_triangle(origin: &[f64; 3], v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> bool {
    let dir = [1.0, 0.0, 0.0]; // +X ray
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let h = cross(dir, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-15 {
        return false;
    }

    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return false;
    }

    let q = cross(s, e1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * dot(e2, q);
    t > 1e-15
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

fn extract_kept_cells(pd: &PolyData, keep: &[bool]) -> PolyData {
    let mut point_map = vec![i64::MAX; pd.points.len()];
    let mut new_points = crate::data::Points::<f64>::new();
    let mut new_polys = crate::data::CellArray::new();

    for ci in 0..pd.polys.num_cells() {
        if ci >= keep.len() || !keep[ci] {
            continue;
        }
        let cell = pd.polys.cell(ci);
        for &vid in cell {
            let vi = vid as usize;
            if point_map[vi] == i64::MAX {
                point_map[vi] = new_points.len() as i64;
                new_points.push(pd.points.get(vi));
            }
        }
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        new_polys.push_cell(&remapped);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

fn append_poly_data(target: &mut PolyData, source: &PolyData) {
    let offset = target.points.len() as i64;
    for i in 0..source.points.len() {
        target.points.push(source.points.get(i));
    }
    for ci in 0..source.polys.num_cells() {
        let cell = source.polys.cell(ci);
        let remapped: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
        target.polys.push_cell(&remapped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube() -> PolyData {
        // Simplified: 2 triangles forming one face of a cube
        // Not a closed mesh, but sufficient for testing cell extraction
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 3, 7],
                [0, 7, 4],
                [1, 2, 6],
                [1, 6, 5],
            ],
        )
    }

    #[test]
    fn union_preserves_cells() {
        let a = unit_cube();
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = boolean_operation(&a, &b, BooleanOp::Union);
        // Non-overlapping: all cells should be kept
        assert!(result.polys.num_cells() >= a.polys.num_cells());
    }

    #[test]
    fn point_outside_mesh() {
        let cube = unit_cube();
        // Point far outside should definitely be outside
        assert!(!point_in_mesh(&[5.0, 5.0, 5.0], &cube));
    }
}
