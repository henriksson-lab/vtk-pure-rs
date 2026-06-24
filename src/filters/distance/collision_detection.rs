//! Collision detection between two triangle meshes.
//!
//! Uses axis-aligned bounding box (AABB) hierarchy for broad-phase culling
//! and triangle-triangle intersection tests for narrow-phase detection.
//! Analogous to VTK's vtkCollisionDetectionFilter.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Result of a collision detection query.
#[derive(Debug, Clone)]
pub struct CollisionResult {
    /// Number of intersecting triangle pairs.
    pub num_contacts: usize,
    /// Indices of intersecting triangles in mesh A.
    pub contacts_a: Vec<usize>,
    /// Indices of intersecting triangles in mesh B.
    pub contacts_b: Vec<usize>,
    /// Whether any collision was detected.
    pub collides: bool,
}

/// Detect collisions between two triangle meshes.
///
/// Returns a `CollisionResult` containing pairs of intersecting triangles.
/// Both meshes must be triangle meshes (3-vertex polygons).
///
/// Uses AABB broad-phase + Möller triangle-triangle narrow-phase.
pub fn collision_detection(mesh_a: &PolyData, mesh_b: &PolyData) -> CollisionResult {
    let tris_a = extract_triangles(mesh_a);
    let tris_b = extract_triangles(mesh_b);

    if tris_a.is_empty() || tris_b.is_empty() {
        return CollisionResult {
            num_contacts: 0,
            contacts_a: Vec::new(),
            contacts_b: Vec::new(),
            collides: false,
        };
    }

    // Build AABB for each triangle
    let aabbs_a: Vec<Aabb> = tris_a.iter().map(|t| tri_aabb(t)).collect();
    let aabbs_b: Vec<Aabb> = tris_b.iter().map(|t| tri_aabb(t)).collect();

    // Global overlap test
    let global_a = merge_aabbs(&aabbs_a);
    let global_b = merge_aabbs(&aabbs_b);
    if !aabb_overlap(&global_a, &global_b) {
        return CollisionResult {
            num_contacts: 0,
            contacts_a: Vec::new(),
            contacts_b: Vec::new(),
            collides: false,
        };
    }

    let mut contacts_a = Vec::new();
    let mut contacts_b = Vec::new();

    // Broad phase: AABB overlap between all pairs
    // (For large meshes, a BVH would be faster, but this is O(n*m) broad phase)
    for (ia, aabb_a) in aabbs_a.iter().enumerate() {
        for (ib, aabb_b) in aabbs_b.iter().enumerate() {
            if !aabb_overlap(aabb_a, aabb_b) {
                continue;
            }
            // Narrow phase: triangle-triangle intersection
            if tri_tri_intersect(&tris_a[ia], &tris_b[ib]) {
                contacts_a.push(ia);
                contacts_b.push(ib);
            }
        }
    }

    let num_contacts = contacts_a.len();
    CollisionResult {
        num_contacts,
        contacts_a,
        contacts_b,
        collides: num_contacts > 0,
    }
}

/// Detect collisions and mark intersecting cells in the output meshes.
///
/// Returns clones of the input meshes with a "CollisionId" cell data array
/// where 1 = intersecting, 0 = not intersecting.
pub fn collision_detection_marked(
    mesh_a: &PolyData,
    mesh_b: &PolyData,
) -> (PolyData, PolyData, CollisionResult) {
    let result = collision_detection(mesh_a, mesh_b);

    let n_cells_a = mesh_a.polys.num_cells();
    let n_cells_b = mesh_b.polys.num_cells();

    let mut marks_a = vec![0.0f64; n_cells_a];
    let mut marks_b = vec![0.0f64; n_cells_b];

    for &idx in &result.contacts_a {
        if idx < n_cells_a {
            marks_a[idx] = 1.0;
        }
    }
    for &idx in &result.contacts_b {
        if idx < n_cells_b {
            marks_b[idx] = 1.0;
        }
    }

    let mut out_a = mesh_a.clone();
    let mut out_b = mesh_b.clone();

    out_a
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CollisionId",
            marks_a,
            1,
        )));
    out_b
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CollisionId",
            marks_b,
            1,
        )));

    (out_a, out_b, result)
}

type Triangle = [[f64; 3]; 3];

fn extract_triangles(mesh: &PolyData) -> Vec<Triangle> {
    let mut tris = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() == 3 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            tris.push([a, b, c]);
        }
    }
    tris
}

#[derive(Clone)]
struct Aabb {
    min: [f64; 3],
    max: [f64; 3],
}

fn tri_aabb(tri: &Triangle) -> Aabb {
    let mut min = tri[0];
    let mut max = tri[0];
    for v in &tri[1..] {
        for i in 0..3 {
            min[i] = min[i].min(v[i]);
            max[i] = max[i].max(v[i]);
        }
    }
    Aabb { min, max }
}

fn merge_aabbs(aabbs: &[Aabb]) -> Aabb {
    let mut result = aabbs[0].clone();
    for aabb in &aabbs[1..] {
        for i in 0..3 {
            result.min[i] = result.min[i].min(aabb.min[i]);
            result.max[i] = result.max[i].max(aabb.max[i]);
        }
    }
    result
}

fn aabb_overlap(a: &Aabb, b: &Aabb) -> bool {
    for i in 0..3 {
        if a.max[i] < b.min[i] || b.max[i] < a.min[i] {
            return false;
        }
    }
    true
}

/// Triangle-triangle intersection test using the separating axis theorem.
fn tri_tri_intersect(t1: &Triangle, t2: &Triangle) -> bool {
    // Edge vectors
    let e1 = [sub(t1[1], t1[0]), sub(t1[2], t1[1]), sub(t1[0], t1[2])];
    let e2 = [sub(t2[1], t2[0]), sub(t2[2], t2[1]), sub(t2[0], t2[2])];

    // Face normals
    let n1 = cross(e1[0], sub(t1[2], t1[0]));
    let n2 = cross(e2[0], sub(t2[2], t2[0]));

    // Test face normals as separating axes
    if separating_axis(t1, t2, n1) {
        return false;
    }
    if separating_axis(t1, t2, n2) {
        return false;
    }

    // Test cross products of edge pairs
    for ea in &e1 {
        for eb in &e2 {
            let axis = cross(*ea, *eb);
            let len_sq = axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2];
            if len_sq < 1e-20 {
                continue;
            } // parallel edges
            if separating_axis(t1, t2, axis) {
                return false;
            }
        }
    }

    true
}

fn separating_axis(t1: &Triangle, t2: &Triangle, axis: [f64; 3]) -> bool {
    let (min1, max1) = project_triangle(t1, axis);
    let (min2, max2) = project_triangle(t2, axis);
    max1 < min2 || max2 < min1
}

fn project_triangle(tri: &Triangle, axis: [f64; 3]) -> (f64, f64) {
    let d0 = dot(tri[0], axis);
    let d1 = dot(tri[1], axis);
    let d2 = dot(tri[2], axis);
    (d0.min(d1).min(d2), d0.max(d1).max(d2))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_triangle(offset: [f64; 3]) -> PolyData {
        PolyData::from_triangles(
            vec![
                [offset[0], offset[1], offset[2]],
                [offset[0] + 1.0, offset[1], offset[2]],
                [offset[0], offset[1] + 1.0, offset[2]],
            ],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn no_collision() {
        let a = make_triangle([0.0, 0.0, 0.0]);
        let b = make_triangle([5.0, 5.0, 0.0]);
        let result = collision_detection(&a, &b);
        assert!(!result.collides);
        assert_eq!(result.num_contacts, 0);
    }

    #[test]
    fn overlapping_triangles() {
        let a = make_triangle([0.0, 0.0, 0.0]);
        let b = make_triangle([0.5, 0.0, 0.0]);
        let result = collision_detection(&a, &b);
        assert!(result.collides);
        assert!(result.num_contacts > 0);
    }

    #[test]
    fn identical_triangles() {
        let a = make_triangle([0.0, 0.0, 0.0]);
        let b = make_triangle([0.0, 0.0, 0.0]);
        let result = collision_detection(&a, &b);
        assert!(result.collides);
    }

    #[test]
    fn marked_output() {
        let a = make_triangle([0.0, 0.0, 0.0]);
        let b = make_triangle([0.5, 0.0, 0.0]);
        let (out_a, out_b, result) = collision_detection_marked(&a, &b);
        assert!(result.collides);
        assert!(out_a.cell_data().get_array("CollisionId").is_some());
        assert!(out_b.cell_data().get_array("CollisionId").is_some());
    }

    #[test]
    fn separated_in_z() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.0, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let result = collision_detection(&a, &b);
        assert!(!result.collides);
    }

    #[test]
    fn multi_triangle_mesh() {
        // Two triangles in mesh A, one overlaps with B
        let a = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let b = make_triangle([0.5, 0.0, 0.0]);
        let result = collision_detection(&a, &b);
        assert!(result.collides);
        assert_eq!(result.num_contacts, 1);
        assert_eq!(result.contacts_a[0], 0); // first triangle of A
    }
}
