use crate::data::{CellArray, Points, PolyData};

/// Compute the 3D convex hull of a point set using an incremental algorithm.
///
/// Starts with an initial tetrahedron from 4 non-coplanar points, then adds
/// remaining points one at a time, removing visible faces and stitching new
/// triangles from the horizon edges to the new point.
///
/// Returns a PolyData with triangle faces forming the convex hull.
pub fn convex_hull_3d(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    if n < 4 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Find 4 non-coplanar points for initial tetrahedron
    let (i0, i1, i2, i3) = match find_initial_tetra(&pts) {
        Some(t) => t,
        None => return PolyData::new(),
    };

    let mut faces: Vec<[usize; 3]> = vec![[i0, i1, i2], [i0, i2, i3], [i0, i3, i1], [i1, i3, i2]];

    // Ensure all initial faces point outward from centroid
    let centroid: [f64; 3] = [
        (pts[i0][0] + pts[i1][0] + pts[i2][0] + pts[i3][0]) / 4.0,
        (pts[i0][1] + pts[i1][1] + pts[i2][1] + pts[i3][1]) / 4.0,
        (pts[i0][2] + pts[i1][2] + pts[i2][2] + pts[i3][2]) / 4.0,
    ];
    for face in &mut faces {
        let nrm: [f64; 3] = face_normal(&pts, face);
        let mid: [f64; 3] = [
            (pts[face[0]][0] + pts[face[1]][0] + pts[face[2]][0]) / 3.0 - centroid[0],
            (pts[face[0]][1] + pts[face[1]][1] + pts[face[2]][1]) / 3.0 - centroid[1],
            (pts[face[0]][2] + pts[face[1]][2] + pts[face[2]][2]) / 3.0 - centroid[2],
        ];
        if dot3(nrm, mid) < 0.0 {
            face.swap(0, 1);
        }
    }

    // Add remaining points incrementally
    let initial: [usize; 4] = [i0, i1, i2, i3];
    for pi in 0..n {
        if initial.contains(&pi) {
            continue;
        }

        // Find visible faces
        let mut visible: Vec<bool> = vec![false; faces.len()];
        let mut any_visible: bool = false;
        for (fi, face) in faces.iter().enumerate() {
            let nrm: [f64; 3] = face_normal(&pts, face);
            let d: [f64; 3] = [
                pts[pi][0] - pts[face[0]][0],
                pts[pi][1] - pts[face[0]][1],
                pts[pi][2] - pts[face[0]][2],
            ];
            if dot3(nrm, d) > 1e-10 {
                visible[fi] = true;
                any_visible = true;
            }
        }

        if !any_visible {
            continue;
        }

        // Find horizon edges
        let mut horizon: Vec<(usize, usize)> = Vec::new();
        for (fi, face) in faces.iter().enumerate() {
            if !visible[fi] {
                continue;
            }
            let edges: [(usize, usize); 3] =
                [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])];
            for (a, b) in edges {
                let mut shared: bool = false;
                for (fj, other) in faces.iter().enumerate() {
                    if fi == fj || visible[fj] {
                        continue;
                    }
                    if face_has_edge(other, b, a) {
                        shared = true;
                        break;
                    }
                }
                if shared {
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

        // Add new faces from horizon to new point
        for (a, b) in &horizon {
            new_faces.push([*a, *b, pi]);
        }

        faces = new_faces;
    }

    // Build output PolyData
    let mut out_points: Points<f64> = Points::new();
    let mut out_polys: CellArray = CellArray::new();

    // Collect unique point indices
    let mut used: Vec<bool> = vec![false; n];
    for face in &faces {
        used[face[0]] = true;
        used[face[1]] = true;
        used[face[2]] = true;
    }

    let mut remap: Vec<i64> = vec![-1; n];
    let mut count: i64 = 0;
    for i in 0..n {
        if used[i] {
            out_points.push(pts[i]);
            remap[i] = count;
            count += 1;
        }
    }

    for face in &faces {
        out_polys.push_cell(&[remap[face[0]], remap[face[1]], remap[face[2]]]);
    }

    let mut output = PolyData::new();
    output.points = out_points;
    output.polys = out_polys;
    output
}

fn face_normal(pts: &[[f64; 3]], face: &[usize; 3]) -> [f64; 3] {
    let a: [f64; 3] = pts[face[0]];
    let b: [f64; 3] = pts[face[1]];
    let c: [f64; 3] = pts[face[2]];
    let u: [f64; 3] = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v: [f64; 3] = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn face_has_edge(face: &[usize; 3], a: usize, b: usize) -> bool {
    (face[0] == a && face[1] == b)
        || (face[1] == a && face[2] == b)
        || (face[2] == a && face[0] == b)
}

fn find_initial_tetra(pts: &[[f64; 3]]) -> Option<(usize, usize, usize, usize)> {
    let n: usize = pts.len();
    if n < 4 {
        return None;
    }

    let i0: usize = 0;

    // Find farthest point from i0
    let mut i1: usize = 1;
    let mut best_dist: f64 = 0.0;
    for i in 1..n {
        let d: f64 = dist_sq(pts[i0], pts[i]);
        if d > best_dist {
            best_dist = d;
            i1 = i;
        }
    }
    if best_dist < 1e-20 {
        return None;
    }

    // Find point farthest from line i0-i1
    let mut i2: usize = 0;
    let mut best_area: f64 = 0.0;
    let dir: [f64; 3] = [
        pts[i1][0] - pts[i0][0],
        pts[i1][1] - pts[i0][1],
        pts[i1][2] - pts[i0][2],
    ];
    for i in 0..n {
        if i == i0 || i == i1 {
            continue;
        }
        let v: [f64; 3] = [
            pts[i][0] - pts[i0][0],
            pts[i][1] - pts[i0][1],
            pts[i][2] - pts[i0][2],
        ];
        let cross: [f64; 3] = [
            dir[1] * v[2] - dir[2] * v[1],
            dir[2] * v[0] - dir[0] * v[2],
            dir[0] * v[1] - dir[1] * v[0],
        ];
        let area: f64 = dot3(cross, cross);
        if area > best_area {
            best_area = area;
            i2 = i;
        }
    }
    if best_area < 1e-20 {
        return None;
    }

    // Find point farthest from plane i0-i1-i2
    let nrm: [f64; 3] = face_normal(pts, &[i0, i1, i2]);
    let mut i3: usize = 0;
    let mut best_vol: f64 = 0.0;
    for i in 0..n {
        if i == i0 || i == i1 || i == i2 {
            continue;
        }
        let v: [f64; 3] = [
            pts[i][0] - pts[i0][0],
            pts[i][1] - pts[i0][1],
            pts[i][2] - pts[i0][2],
        ];
        let vol: f64 = dot3(nrm, v).abs();
        if vol > best_vol {
            best_vol = vol;
            i3 = i;
        }
    }
    if best_vol < 1e-20 {
        return None;
    }

    Some((i0, i1, i2, i3))
}

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx: f64 = a[0] - b[0];
    let dy: f64 = a[1] - b[1];
    let dz: f64 = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cube_points() -> PolyData {
        let mut pd = PolyData::new();
        for &x in &[-1.0f64, 1.0] {
            for &y in &[-1.0f64, 1.0] {
                for &z in &[-1.0f64, 1.0] {
                    pd.points.push([x, y, z]);
                }
            }
        }
        pd
    }

    #[test]
    fn hull_of_cube_has_correct_face_count() {
        let pd = make_cube_points();
        let hull = convex_hull_3d(&pd);
        // A cube convex hull should have 12 triangles (6 faces * 2 triangles each)
        assert_eq!(hull.polys.num_cells(), 12);
        assert_eq!(hull.points.len(), 8);
    }

    #[test]
    fn hull_of_tetrahedron() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        let hull = convex_hull_3d(&pd);
        assert_eq!(hull.polys.num_cells(), 4);
        assert_eq!(hull.points.len(), 4);
    }

    #[test]
    fn hull_with_interior_point() {
        let mut pd = PolyData::new();
        // Tetrahedron vertices
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([1.0, 1.0, 2.0]);
        // Interior point
        pd.points.push([1.0, 0.8, 0.5]);
        let hull = convex_hull_3d(&pd);
        // Interior point should not add faces; still 4 triangles
        assert_eq!(hull.polys.num_cells(), 4);
        // Interior point should not appear in the hull points
        assert_eq!(hull.points.len(), 4);
    }
}
