use crate::data::{CellArray, Points, PolyData};

/// Approximate boolean union of two triangle meshes.
///
/// Combines both meshes, then removes faces from each that are "inside"
/// the other using a winding-number test on the face centroid.
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
        if cell.len() < 3 {
            continue;
        }
        let p0 = pd.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = pd.points.get(cell[i] as usize);
            let p2 = pd.points.get(cell[i + 1] as usize);
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

/// Generalized winding-number test. Values near one are inside a consistently
/// oriented closed surface, values near zero are outside.
fn point_inside_mesh(point: &[f64; 3], tris: &[Triangle]) -> bool {
    let mut angle = 0.0;
    for (v0, v1, v2) in tris {
        angle += solid_angle(*point, [*v0, *v1, *v2]);
    }
    (angle / (4.0 * std::f64::consts::PI)).abs() > 0.5
}

fn solid_angle(p: [f64; 3], tri: [[f64; 3]; 3]) -> f64 {
    let a = sub(tri[0], p);
    let b = sub(tri[1], p);
    let c = sub(tri[2], p);
    let la = norm(a);
    let lb = norm(b);
    let lc = norm(c);
    if la < 1e-15 || lb < 1e-15 || lc < 1e-15 {
        return 0.0;
    }

    let det = dot(a, cross(b, c));
    let denom = la * lb * lc + dot(a, b) * lc + dot(a, c) * lb + dot(b, c) * la;
    if denom.abs() < 1e-15 {
        return 0.0;
    }
    2.0 * det.atan2(denom)
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
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
