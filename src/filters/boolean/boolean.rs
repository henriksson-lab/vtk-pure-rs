use crate::data::{CellArray, Points, PolyData};

/// Boolean operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    Union,
    Intersection,
    Difference,
}

/// Perform a Boolean operation on two closed triangle meshes.
///
/// Classifies each triangle of each mesh as inside or outside the other mesh
/// using ray-casting at the centroid. Then selects the appropriate triangles:
///
/// - **Union**: outside(A) + outside(B)
/// - **Intersection**: inside(A) + inside(B)
/// - **Difference** (A - B): outside_of_B(A) + inside_of_A(B) with reversed winding
///
/// Both inputs should be closed (watertight) triangle meshes for correct results.
pub fn boolean(a: &PolyData, b: &PolyData, op: BooleanOp) -> PolyData {
    let tris_a = collect_triangles(a);
    let tris_b = collect_triangles(b);

    // Classify A triangles w.r.t. B
    let a_inside_b: Vec<bool> = tris_a
        .iter()
        .map(|tri| {
            let c = centroid(tri);
            is_inside(&tris_b, c)
        })
        .collect();

    // Classify B triangles w.r.t. A
    let b_inside_a: Vec<bool> = tris_b
        .iter()
        .map(|tri| {
            let c = centroid(tri);
            is_inside(&tris_a, c)
        })
        .collect();

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    match op {
        BooleanOp::Union => {
            // Keep A triangles outside B
            for (i, tri) in tris_a.iter().enumerate() {
                if !a_inside_b[i] {
                    add_triangle(&mut out_points, &mut out_polys, tri, false);
                }
            }
            // Keep B triangles outside A
            for (i, tri) in tris_b.iter().enumerate() {
                if !b_inside_a[i] {
                    add_triangle(&mut out_points, &mut out_polys, tri, false);
                }
            }
        }
        BooleanOp::Intersection => {
            // Keep A triangles inside B
            for (i, tri) in tris_a.iter().enumerate() {
                if a_inside_b[i] {
                    add_triangle(&mut out_points, &mut out_polys, tri, false);
                }
            }
            // Keep B triangles inside A
            for (i, tri) in tris_b.iter().enumerate() {
                if b_inside_a[i] {
                    add_triangle(&mut out_points, &mut out_polys, tri, false);
                }
            }
        }
        BooleanOp::Difference => {
            // Keep A triangles outside B
            for (i, tri) in tris_a.iter().enumerate() {
                if !a_inside_b[i] {
                    add_triangle(&mut out_points, &mut out_polys, tri, false);
                }
            }
            // Keep B triangles inside A, but reverse winding
            for (i, tri) in tris_b.iter().enumerate() {
                if b_inside_a[i] {
                    add_triangle(&mut out_points, &mut out_polys, tri, true);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// A triangle with 3 vertex positions.
type Tri = [[f64; 3]; 3];

fn collect_triangles(pd: &PolyData) -> Vec<Tri> {
    let mut tris = Vec::new();
    for cell in pd.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let p0 = pd.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = pd.points.get(cell[i] as usize);
            let p2 = pd.points.get(cell[i + 1] as usize);
            tris.push([
                [p0[0], p0[1], p0[2]],
                [p1[0], p1[1], p1[2]],
                [p2[0], p2[1], p2[2]],
            ]);
        }
    }
    tris
}

fn centroid(tri: &Tri) -> [f64; 3] {
    [
        (tri[0][0] + tri[1][0] + tri[2][0]) / 3.0,
        (tri[0][1] + tri[1][1] + tri[2][1]) / 3.0,
        (tri[0][2] + tri[1][2] + tri[2][2]) / 3.0,
    ]
}

/// Ray-casting inside/outside test. Casts a ray in +X from the point and
/// counts triangle intersections. Odd count = inside.
fn is_inside(tris: &[Tri], point: [f64; 3]) -> bool {
    let mut count = 0u32;
    for tri in tris {
        if ray_hits_triangle(point, tri) {
            count += 1;
        }
    }
    count % 2 == 1
}

/// Möller–Trumbore ray-triangle intersection (+X direction).
fn ray_hits_triangle(origin: [f64; 3], tri: &Tri) -> bool {
    let e1 = sub(tri[1], tri[0]);
    let e2 = sub(tri[2], tri[0]);
    let dir = [1.0, 0.0, 0.0];

    let h = cross(dir, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-12 {
        return false;
    }

    let f = 1.0 / a;
    let s = sub(origin, tri[0]);
    let u = f * dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return false;
    }

    let q = cross(s, e1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * dot(e2, q);
    t > 1e-12
}

fn add_triangle(points: &mut Points<f64>, polys: &mut CellArray, tri: &Tri, reverse: bool) {
    let base = points.len() as i64;
    points.push(tri[0]);
    points.push(tri[1]);
    points.push(tri[2]);
    if reverse {
        polys.push_cell(&[base, base + 2, base + 1]);
    } else {
        polys.push_cell(&[base, base + 1, base + 2]);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a simple axis-aligned box as a closed triangle mesh.
    fn make_box(min: [f64; 3], max: [f64; 3]) -> PolyData {
        let mut pd = PolyData::new();
        // 8 corners
        let corners = [
            [min[0], min[1], min[2]], // 0
            [max[0], min[1], min[2]], // 1
            [max[0], max[1], min[2]], // 2
            [min[0], max[1], min[2]], // 3
            [min[0], min[1], max[2]], // 4
            [max[0], min[1], max[2]], // 5
            [max[0], max[1], max[2]], // 6
            [min[0], max[1], max[2]], // 7
        ];
        for c in &corners {
            pd.points.push(*c);
        }

        // 6 faces as triangle pairs (outward normals)
        let faces: [[usize; 4]; 6] = [
            [0, 3, 2, 1], // -Z
            [4, 5, 6, 7], // +Z
            [0, 1, 5, 4], // -Y
            [2, 3, 7, 6], // +Y
            [0, 4, 7, 3], // -X
            [1, 2, 6, 5], // +X
        ];

        for face in &faces {
            pd.polys
                .push_cell(&[face[0] as i64, face[1] as i64, face[2] as i64]);
            pd.polys
                .push_cell(&[face[0] as i64, face[2] as i64, face[3] as i64]);
        }
        pd
    }

    #[test]
    fn union_disjoint() {
        let a = make_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_box([3.0, 0.0, 0.0], [4.0, 1.0, 1.0]);
        let result = boolean(&a, &b, BooleanOp::Union);
        // Disjoint: union = all triangles from both
        assert_eq!(result.polys.num_cells(), 24); // 12 + 12
    }

    #[test]
    fn intersection_disjoint() {
        let a = make_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_box([3.0, 0.0, 0.0], [4.0, 1.0, 1.0]);
        let result = boolean(&a, &b, BooleanOp::Intersection);
        // Disjoint: intersection = empty
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn difference_disjoint() {
        let a = make_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_box([3.0, 0.0, 0.0], [4.0, 1.0, 1.0]);
        let result = boolean(&a, &b, BooleanOp::Difference);
        // Disjoint: A - B = A
        assert_eq!(result.polys.num_cells(), 12);
    }

    #[test]
    fn union_overlapping() {
        let a = make_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_box([0.5, 0.0, 0.0], [1.5, 1.0, 1.0]);
        let result = boolean(&a, &b, BooleanOp::Union);
        // Overlapping: some triangles from each are inside the other
        assert!(result.polys.num_cells() > 0);
        assert!(result.polys.num_cells() < 24);
    }

    #[test]
    fn intersection_overlapping() {
        let a = make_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_box([0.5, 0.0, 0.0], [1.5, 1.0, 1.0]);
        let result = boolean(&a, &b, BooleanOp::Intersection);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn difference_overlapping() {
        let a = make_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = make_box([0.5, 0.0, 0.0], [1.5, 1.0, 1.0]);
        let result = boolean(&a, &b, BooleanOp::Difference);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn contained_intersection() {
        // B fully inside A
        let a = make_box([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_box([0.5, 0.5, 0.5], [1.5, 1.5, 1.5]);
        let result = boolean(&a, &b, BooleanOp::Intersection);
        // All of B is inside A, none of A is inside B
        assert_eq!(result.polys.num_cells(), 12); // just B
    }

    #[test]
    fn contained_union() {
        let a = make_box([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let b = make_box([0.5, 0.5, 0.5], [1.5, 1.5, 1.5]);
        let result = boolean(&a, &b, BooleanOp::Union);
        // All of A is outside B, none of B is outside A
        assert_eq!(result.polys.num_cells(), 12); // just A
    }
}
