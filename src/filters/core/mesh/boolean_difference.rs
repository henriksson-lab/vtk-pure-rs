use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Simple boolean difference approximation: remove from mesh A all triangles
/// whose centroids are inside mesh B (using ray-casting for inside/outside test).
///
/// This is an approximation that works on closed, watertight meshes. It does not
/// compute exact intersection curves.
pub fn boolean_difference(a: &PolyData, b: &PolyData) -> PolyData {
    // Precompute face normals and face data for mesh B's inside/outside test
    let b_tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = collect_triangles(b);

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut point_map: HashMap<usize, usize> = HashMap::new();

    for cell in a.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Compute centroid of this face
        let mut cx: f64 = 0.0;
        let mut cy: f64 = 0.0;
        let mut cz: f64 = 0.0;
        let count: f64 = cell.len() as f64;
        for &idx in cell.iter() {
            let p = a.points.get(idx as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        cx /= count;
        cy /= count;
        cz /= count;

        // If centroid is inside mesh B, skip this face (subtract it)
        if is_inside_mesh([cx, cy, cz], &b_tris) {
            continue;
        }

        // Map and add points
        let mut new_cell: Vec<i64> = Vec::with_capacity(cell.len());
        for &idx in cell.iter() {
            let orig: usize = idx as usize;
            let mapped: usize = *point_map.entry(orig).or_insert_with(|| {
                let new_idx: usize = out_points.len();
                out_points.push(a.points.get(orig));
                new_idx
            });
            new_cell.push(mapped as i64);
        }
        out_polys.push_cell(&new_cell);
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.polys = out_polys;
    result
}

fn collect_triangles(pd: &PolyData) -> Vec<([f64; 3], [f64; 3], [f64; 3])> {
    let mut tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = Vec::new();
    for cell in pd.polys.iter() {
        if cell.len() >= 3 {
            let v0 = pd.points.get(cell[0] as usize);
            // Fan-triangulate polygons
            for i in 1..cell.len() - 1 {
                let v1 = pd.points.get(cell[i] as usize);
                let v2 = pd.points.get(cell[i + 1] as usize);
                tris.push((v0, v1, v2));
            }
        }
    }
    tris
}

/// Ray-casting inside/outside test. Cast a ray in +X direction from point and
/// count intersections with triangles.
fn is_inside_mesh(point: [f64; 3], tris: &[([f64; 3], [f64; 3], [f64; 3])]) -> bool {
    let mut intersections: Vec<f64> = Vec::new();
    let origin = point;
    let dir: [f64; 3] = [1.0, 0.0, 0.0]; // +X ray

    for &(v0, v1, v2) in tris {
        if let Some(t) = ray_triangle_intersect(origin, dir, v0, v1, v2) {
            if !intersections.iter().any(|&prev| (prev - t).abs() < 1e-9) {
                intersections.push(t);
            }
        }
    }

    // Odd number of crossings means inside
    intersections.len() % 2 == 1
}

/// Moller-Trumbore ray-triangle intersection test.
fn ray_triangle_intersect(
    origin: [f64; 3],
    dir: [f64; 3],
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
) -> Option<f64> {
    let eps: f64 = 1e-10;

    let e1: [f64; 3] = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2: [f64; 3] = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let h: [f64; 3] = cross(dir, e2);
    let a: f64 = dot(e1, h);

    if a.abs() < eps {
        return None;
    }

    let f: f64 = 1.0 / a;
    let s: [f64; 3] = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u: f64 = f * dot(s, h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q: [f64; 3] = cross(s, e1);
    let v: f64 = f * dot(dir, q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t: f64 = f * dot(e2, q);
    if t > eps {
        Some(t)
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple axis-aligned box from (-s,-s,-s) to (s,s,s).
    fn make_box(s: f64) -> PolyData {
        let mut pd = PolyData::new();
        // 8 vertices of a cube
        pd.points.push([-s, -s, -s]); // 0
        pd.points.push([s, -s, -s]); // 1
        pd.points.push([s, s, -s]); // 2
        pd.points.push([-s, s, -s]); // 3
        pd.points.push([-s, -s, s]); // 4
        pd.points.push([s, -s, s]); // 5
        pd.points.push([s, s, s]); // 6
        pd.points.push([-s, s, s]); // 7

        // 12 triangles (2 per face)
        // Front face (z = s)
        pd.polys.push_cell(&[4, 5, 6]);
        pd.polys.push_cell(&[4, 6, 7]);
        // Back face (z = -s)
        pd.polys.push_cell(&[1, 0, 3]);
        pd.polys.push_cell(&[1, 3, 2]);
        // Right face (x = s)
        pd.polys.push_cell(&[1, 2, 6]);
        pd.polys.push_cell(&[1, 6, 5]);
        // Left face (x = -s)
        pd.polys.push_cell(&[0, 4, 7]);
        pd.polys.push_cell(&[0, 7, 3]);
        // Top face (y = s)
        pd.polys.push_cell(&[3, 7, 6]);
        pd.polys.push_cell(&[3, 6, 2]);
        // Bottom face (y = -s)
        pd.polys.push_cell(&[0, 1, 5]);
        pd.polys.push_cell(&[0, 5, 4]);
        pd
    }

    #[test]
    fn difference_with_non_overlapping_keeps_all() {
        let a = make_box(1.0);
        // b is far away, no overlap
        let mut b = make_box(0.5);
        for i in 0..b.points.len() {
            let p = b.points.get(i);
            b.points.set(i, [p[0] + 10.0, p[1], p[2]]);
        }
        let result = boolean_difference(&a, &b);
        assert_eq!(result.polys.num_cells(), a.polys.num_cells());
    }

    #[test]
    fn difference_removes_interior_faces() {
        let a = make_box(0.5);
        let b = make_box(2.0);
        // a is entirely inside b, so all faces of a should be removed
        let result = boolean_difference(&a, &b);
        assert_eq!(
            result.polys.num_cells(),
            0,
            "all faces of a should be inside b"
        );
    }

    #[test]
    fn difference_with_empty_b_keeps_all() {
        let a = make_box(1.0);
        let b = PolyData::new();
        let result = boolean_difference(&a, &b);
        assert_eq!(result.polys.num_cells(), a.polys.num_cells());
    }
}
