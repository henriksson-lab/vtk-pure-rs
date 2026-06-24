use crate::data::{CellArray, Points, PolyData};

/// Compute the 3D convex hull of a point set.
///
/// Uses an incremental algorithm: starts with a tetrahedron from 4 non-coplanar
/// points, then adds remaining points one at a time, removing visible faces
/// and creating new ones.
///
/// Returns a PolyData with triangle faces forming the convex hull.
pub fn convex_hull(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 4 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Find 4 non-coplanar points for initial tetrahedron
    let (i0, i1, i2, i3) = match find_initial_tetra(&pts) {
        Some(t) => t,
        None => return PolyData::new(), // All points coplanar
    };

    // Faces are stored as (a, b, c) with outward-facing normal
    let mut faces: Vec<[usize; 3]> = vec![[i0, i1, i2], [i0, i2, i3], [i0, i3, i1], [i1, i3, i2]];

    // Ensure all initial faces point outward
    let centroid = [
        (pts[i0][0] + pts[i1][0] + pts[i2][0] + pts[i3][0]) / 4.0,
        (pts[i0][1] + pts[i1][1] + pts[i2][1] + pts[i3][1]) / 4.0,
        (pts[i0][2] + pts[i1][2] + pts[i2][2] + pts[i3][2]) / 4.0,
    ];
    for face in &mut faces {
        let n = face_normal(&pts, face);
        let mid = [
            (pts[face[0]][0] + pts[face[1]][0] + pts[face[2]][0]) / 3.0 - centroid[0],
            (pts[face[0]][1] + pts[face[1]][1] + pts[face[2]][1]) / 3.0 - centroid[1],
            (pts[face[0]][2] + pts[face[1]][2] + pts[face[2]][2]) / 3.0 - centroid[2],
        ];
        if dot3(n, mid) < 0.0 {
            face.swap(0, 1);
        }
    }

    // Add remaining points incrementally
    let initial = [i0, i1, i2, i3];
    for pi in 0..n {
        if initial.contains(&pi) {
            continue;
        }

        // Find visible faces
        let mut visible = vec![false; faces.len()];
        let mut any_visible = false;
        for (fi, face) in faces.iter().enumerate() {
            let n = face_normal(&pts, face);
            let d = [
                pts[pi][0] - pts[face[0]][0],
                pts[pi][1] - pts[face[0]][1],
                pts[pi][2] - pts[face[0]][2],
            ];
            if dot3(n, d) > 1e-10 {
                visible[fi] = true;
                any_visible = true;
            }
        }

        if !any_visible {
            continue; // Point is inside hull
        }

        // Find horizon edges (edges between visible and non-visible faces)
        let mut horizon: Vec<(usize, usize)> = Vec::new();
        for (fi, face) in faces.iter().enumerate() {
            if !visible[fi] {
                continue;
            }
            let edges = [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])];
            for &(a, b) in &edges {
                // Check if the adjacent face (sharing edge b,a) is non-visible
                let is_horizon = faces.iter().enumerate().any(|(fj, other)| {
                    if fj == fi || visible[fj] {
                        return false;
                    }
                    let oedges = [
                        (other[0], other[1]),
                        (other[1], other[2]),
                        (other[2], other[0]),
                    ];
                    oedges.contains(&(b, a))
                });
                if is_horizon {
                    horizon.push((a, b));
                }
            }
        }

        // Remove visible faces
        let mut new_faces: Vec<[usize; 3]> = Vec::new();
        for (fi, face) in faces.iter().enumerate() {
            if !visible[fi] {
                new_faces.push(*face);
            }
        }

        // Add new faces connecting horizon edges to the new point
        for &(a, b) in &horizon {
            new_faces.push([a, b, pi]);
        }

        faces = new_faces;
    }

    // Build output PolyData
    let mut out_points = Points::<f64>::new();
    for pt in &pts {
        out_points.push(*pt);
    }

    let mut out_polys = CellArray::new();
    for face in &faces {
        out_polys.push_cell(&[face[0] as i64, face[1] as i64, face[2] as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn find_initial_tetra(pts: &[[f64; 3]]) -> Option<(usize, usize, usize, usize)> {
    let n = pts.len();
    let i0 = 0;

    // Find point farthest from i0
    let i1 = (1..n).max_by(|&a, &b| {
        dist2(pts[i0], pts[a])
            .partial_cmp(&dist2(pts[i0], pts[b]))
            .unwrap()
    })?;

    // Find point farthest from line i0-i1
    let dir = sub3(pts[i1], pts[i0]);
    let i2 = (0..n).filter(|&i| i != i0 && i != i1).max_by(|&a, &b| {
        point_line_dist2(pts[a], pts[i0], dir)
            .partial_cmp(&point_line_dist2(pts[b], pts[i0], dir))
            .unwrap()
    })?;

    // Find point farthest from plane i0-i1-i2
    let normal = cross3(sub3(pts[i1], pts[i0]), sub3(pts[i2], pts[i0]));
    if length3(normal) < 1e-20 {
        return None; // Collinear
    }

    let i3 = (0..n)
        .filter(|&i| i != i0 && i != i1 && i != i2)
        .max_by(|&a, &b| {
            let da = dot3(sub3(pts[a], pts[i0]), normal).abs();
            let db = dot3(sub3(pts[b], pts[i0]), normal).abs();
            da.partial_cmp(&db).unwrap()
        })?;

    let d = dot3(sub3(pts[i3], pts[i0]), normal).abs();
    if d < 1e-10 {
        return None; // Coplanar
    }

    Some((i0, i1, i2, i3))
}

fn face_normal(pts: &[[f64; 3]], face: &[usize; 3]) -> [f64; 3] {
    cross3(
        sub3(pts[face[1]], pts[face[0]]),
        sub3(pts[face[2]], pts[face[0]]),
    )
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub3(a, b);
    dot3(d, d)
}

fn point_line_dist2(p: [f64; 3], origin: [f64; 3], dir: [f64; 3]) -> f64 {
    let v = sub3(p, origin);
    let proj = dot3(v, dir) / dot3(dir, dir).max(1e-30);
    let closest = [
        origin[0] + proj * dir[0],
        origin[1] + proj * dir[1],
        origin[2] + proj * dir[2],
    ];
    dist2(p, closest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_of_cube_vertices() {
        let mut pd = PolyData::new();
        // 8 corners of a unit cube
        for &z in &[0.0, 1.0] {
            for &y in &[0.0, 1.0] {
                for &x in &[0.0, 1.0] {
                    pd.points.push([x, y, z]);
                }
            }
        }
        let result = convex_hull(&pd);
        // Cube hull has 12 triangles (6 faces × 2 tris each)
        assert_eq!(result.polys.num_cells(), 12);
    }

    #[test]
    fn hull_of_tetra() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        let result = convex_hull(&pd);
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn hull_with_interior_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([1.0, 1.0, 2.0]);
        pd.points.push([1.0, 0.5, 0.5]); // Interior point
        let result = convex_hull(&pd);
        // Interior point should not add faces — still 4 triangles
        assert_eq!(result.polys.num_cells(), 4);
    }
}
