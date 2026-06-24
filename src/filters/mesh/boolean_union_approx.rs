use crate::data::{CellArray, Points, PolyData};

/// Approximate boolean union of two triangle meshes.
///
/// Combines both meshes, then removes faces from each that are "inside"
/// the other using a ray-casting parity test on the face centroid.
/// Both inputs should be closed, manifold triangle meshes for best results.
pub fn boolean_union_approx(a: &PolyData, b: &PolyData) -> PolyData {
    let faces_a = collect_triangles(a);
    let faces_b = collect_triangles(b);

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Keep faces of A that are NOT inside B
    for (p0, p1, p2) in &faces_a {
        let centroid = tri_centroid(p0, p1, p2);
        if !point_inside_mesh(&centroid, &faces_b) {
            let base: i64 = out_points.len() as i64;
            out_points.push(*p0);
            out_points.push(*p1);
            out_points.push(*p2);
            out_polys.push_cell(&[base, base + 1, base + 2]);
        }
    }

    // Keep faces of B that are NOT inside A
    for (p0, p1, p2) in &faces_b {
        let centroid = tri_centroid(p0, p1, p2);
        if !point_inside_mesh(&centroid, &faces_a) {
            let base: i64 = out_points.len() as i64;
            out_points.push(*p0);
            out_points.push(*p1);
            out_points.push(*p2);
            out_polys.push_cell(&[base, base + 1, base + 2]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

type Triangle = ([f64; 3], [f64; 3], [f64; 3]);

fn collect_triangles(pd: &PolyData) -> Vec<Triangle> {
    let mut tris = Vec::new();
    for cell in pd.polys.iter() {
        if cell.len() >= 3 {
            let p0 = pd.points.get(cell[0] as usize);
            let p1 = pd.points.get(cell[1] as usize);
            let p2 = pd.points.get(cell[2] as usize);
            tris.push((p0, p1, p2));
        }
    }
    tris
}

fn tri_centroid(p0: &[f64; 3], p1: &[f64; 3], p2: &[f64; 3]) -> [f64; 3] {
    [
        (p0[0] + p1[0] + p2[0]) / 3.0,
        (p0[1] + p1[1] + p2[1]) / 3.0,
        (p0[2] + p1[2] + p2[2]) / 3.0,
    ]
}

/// Ray-casting test: shoot a ray in +X direction from `point` and count
/// intersections with the mesh triangles. Odd count = inside.
fn point_inside_mesh(point: &[f64; 3], tris: &[Triangle]) -> bool {
    let mut count: u32 = 0;
    for (v0, v1, v2) in tris {
        if ray_intersects_triangle(point, v0, v1, v2) {
            count += 1;
        }
    }
    count % 2 == 1
}

/// Moller-Trumbore ray-triangle intersection for ray origin `o` in direction +X.
fn ray_intersects_triangle(o: &[f64; 3], v0: &[f64; 3], v1: &[f64; 3], v2: &[f64; 3]) -> bool {
    let dir = [1.0, 0.0, 0.0]; // +X ray direction
    let eps: f64 = 1e-12;

    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    // h = dir x e2
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];

    let a: f64 = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < eps {
        return false;
    }

    let f: f64 = 1.0 / a;
    let s = [o[0] - v0[0], o[1] - v0[1], o[2] - v0[2]];
    let u: f64 = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }

    // q = s x e1
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];

    let v: f64 = f * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t: f64 = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > eps
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an axis-aligned box as 12 triangles centered at `center` with half-size `h`.
    fn make_box(center: [f64; 3], h: f64) -> PolyData {
        let cx = center[0];
        let cy = center[1];
        let cz = center[2];
        let verts: Vec<[f64; 3]> = vec![
            [cx - h, cy - h, cz - h], // 0
            [cx + h, cy - h, cz - h], // 1
            [cx + h, cy + h, cz - h], // 2
            [cx - h, cy + h, cz - h], // 3
            [cx - h, cy - h, cz + h], // 4
            [cx + h, cy - h, cz + h], // 5
            [cx + h, cy + h, cz + h], // 6
            [cx - h, cy + h, cz + h], // 7
        ];
        let faces: Vec<[i64; 3]> = vec![
            // -Z face
            [0, 2, 1],
            [0, 3, 2],
            // +Z face
            [4, 5, 6],
            [4, 6, 7],
            // -Y face
            [0, 1, 5],
            [0, 5, 4],
            // +Y face
            [2, 3, 7],
            [2, 7, 6],
            // -X face
            [0, 4, 7],
            [0, 7, 3],
            // +X face
            [1, 2, 6],
            [1, 6, 5],
        ];
        PolyData::from_triangles(verts, faces)
    }

    #[test]
    fn union_non_overlapping() {
        let a = make_box([0.0, 0.0, 0.0], 0.5);
        let b = make_box([3.0, 0.0, 0.0], 0.5);
        let result = boolean_union_approx(&a, &b);
        // No overlap, so all 24 faces should be kept
        assert_eq!(result.polys.num_cells(), 24);
    }

    #[test]
    fn union_fully_contained() {
        let outer = make_box([0.0, 0.0, 0.0], 2.0);
        let inner = make_box([0.0, 0.0, 0.0], 0.5);
        let result = boolean_union_approx(&outer, &inner);
        // Inner box is fully inside outer, so inner faces should be removed.
        // Outer faces remain (12), inner faces removed (0).
        assert_eq!(result.polys.num_cells(), 12);
    }

    #[test]
    fn union_overlapping() {
        let a = make_box([0.0, 0.0, 0.0], 1.0);
        let b = make_box([1.0, 0.0, 0.0], 1.0);
        let result = boolean_union_approx(&a, &b);
        // Some faces from each should be removed (those inside the other)
        let total: usize = result.polys.num_cells();
        assert!(total < 24, "some faces should be removed, got {}", total);
        assert!(total > 0, "should have some faces");
    }
}
