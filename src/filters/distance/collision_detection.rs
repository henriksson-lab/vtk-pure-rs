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
    let aabbs_a: Vec<Aabb> = tris_a.iter().map(|(_, t)| tri_aabb(t)).collect();
    let aabbs_b: Vec<Aabb> = tris_b.iter().map(|(_, t)| tri_aabb(t)).collect();

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
            if tri_tri_intersect(&tris_a[ia].1, &tris_b[ib].1) {
                contacts_a.push(tris_a[ia].0);
                contacts_b.push(tris_b[ib].0);
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

fn extract_triangles(mesh: &PolyData) -> Vec<(usize, Triangle)> {
    let mut tris = Vec::new();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if cell.len() == 3 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            tris.push((cell_id, [a, b, c]));
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

/// Triangle-triangle intersection test.
fn tri_tri_intersect(t1: &Triangle, t2: &Triangle) -> bool {
    let n1 = cross(sub(t1[1], t1[0]), sub(t1[2], t1[0]));
    let n2 = cross(sub(t2[1], t2[0]), sub(t2[2], t2[0]));

    if norm2(n1) < 1e-20 || norm2(n2) < 1e-20 {
        return false;
    }

    if are_coplanar(t1, t2, n1, n2) {
        return coplanar_tri_tri_intersect(t1, t2, n1);
    }

    // Edge vectors
    let e1 = [sub(t1[1], t1[0]), sub(t1[2], t1[1]), sub(t1[0], t1[2])];
    let e2 = [sub(t2[1], t2[0]), sub(t2[2], t2[1]), sub(t2[0], t2[2])];

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

fn are_coplanar(t1: &Triangle, t2: &Triangle, n1: [f64; 3], n2: [f64; 3]) -> bool {
    let n_cross = cross(n1, n2);
    if norm2(n_cross) > 1e-20 * norm2(n1) * norm2(n2) {
        return false;
    }

    let plane_offset = dot(n1, t1[0]);
    t2.iter()
        .all(|&p| (dot(n1, p) - plane_offset).abs() <= 1e-10 * norm2(n1).sqrt())
}

fn coplanar_tri_tri_intersect(t1: &Triangle, t2: &Triangle, normal: [f64; 3]) -> bool {
    let axis = dominant_axis(normal);
    let a = [
        project_2d(t1[0], axis),
        project_2d(t1[1], axis),
        project_2d(t1[2], axis),
    ];
    let b = [
        project_2d(t2[0], axis),
        project_2d(t2[1], axis),
        project_2d(t2[2], axis),
    ];

    for i in 0..3 {
        let a0 = a[i];
        let a1 = a[(i + 1) % 3];
        for j in 0..3 {
            let b0 = b[j];
            let b1 = b[(j + 1) % 3];
            if segments_intersect_2d(a0, a1, b0, b1) {
                return true;
            }
        }
    }

    point_in_triangle_2d(a[0], &b) || point_in_triangle_2d(b[0], &a)
}

fn dominant_axis(normal: [f64; 3]) -> usize {
    let ax = normal[0].abs();
    let ay = normal[1].abs();
    let az = normal[2].abs();
    if ax > ay && ax > az {
        0
    } else if ay > az {
        1
    } else {
        2
    }
}

fn project_2d(p: [f64; 3], drop_axis: usize) -> [f64; 2] {
    match drop_axis {
        0 => [p[1], p[2]],
        1 => [p[0], p[2]],
        _ => [p[0], p[1]],
    }
}

fn segments_intersect_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    let o1 = orient_2d(a, b, c);
    let o2 = orient_2d(a, b, d);
    let o3 = orient_2d(c, d, a);
    let o4 = orient_2d(c, d, b);
    let eps = 1e-12;

    if o1.abs() <= eps && on_segment_2d(a, b, c) {
        return true;
    }
    if o2.abs() <= eps && on_segment_2d(a, b, d) {
        return true;
    }
    if o3.abs() <= eps && on_segment_2d(c, d, a) {
        return true;
    }
    if o4.abs() <= eps && on_segment_2d(c, d, b) {
        return true;
    }

    (o1 > 0.0) != (o2 > 0.0) && (o3 > 0.0) != (o4 > 0.0)
}

fn orient_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn on_segment_2d(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> bool {
    let eps = 1e-12;
    p[0] >= a[0].min(b[0]) - eps
        && p[0] <= a[0].max(b[0]) + eps
        && p[1] >= a[1].min(b[1]) - eps
        && p[1] <= a[1].max(b[1]) + eps
}

fn point_in_triangle_2d(p: [f64; 2], tri: &[[f64; 2]; 3]) -> bool {
    let d0 = orient_2d(tri[0], tri[1], p);
    let d1 = orient_2d(tri[1], tri[2], p);
    let d2 = orient_2d(tri[2], tri[0], p);
    let eps = 1e-12;
    let has_neg = d0 < -eps || d1 < -eps || d2 < -eps;
    let has_pos = d0 > eps || d1 > eps || d2 > eps;
    !(has_neg && has_pos)
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

fn norm2(a: [f64; 3]) -> f64 {
    dot(a, a)
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
    fn coplanar_triangles_with_overlapping_aabbs_but_no_contact() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[1.5, 1.5, 0.0], [2.5, 1.5, 0.0], [1.5, 2.5, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = collision_detection(&a, &b);

        assert!(!result.collides);
        assert_eq!(result.num_contacts, 0);
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

    #[test]
    fn reports_original_cell_ids_when_non_triangles_are_skipped() {
        let a = PolyData::from_polygons(
            vec![
                [10.0, 10.0, 10.0],
                [11.0, 10.0, 10.0],
                [11.0, 11.0, 10.0],
                [10.0, 11.0, 10.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3], vec![4, 5, 6]],
        );
        let b = make_triangle([0.5, 0.0, 0.0]);

        let result = collision_detection(&a, &b);

        assert!(result.collides);
        assert_eq!(result.contacts_a, vec![1]);
        assert_eq!(result.contacts_b, vec![0]);
    }
}
