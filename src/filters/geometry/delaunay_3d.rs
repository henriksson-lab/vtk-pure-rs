use crate::data::{PolyData, UnstructuredGrid};
use crate::types::CellType;

/// Compute a 3D Delaunay tetrahedralization from a set of points.
///
/// Takes a PolyData containing points and produces an UnstructuredGrid
/// of tetrahedra. Uses the Bowyer-Watson algorithm with a super-tetrahedron.
pub fn delaunay_3d(input: &PolyData) -> UnstructuredGrid {
    let n = input.points.len();
    if n < 4 {
        return UnstructuredGrid::new();
    }

    // Extract 3D coordinates
    let pts: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();

    // Compute bounding box
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for p in &pts {
        for k in 0..3 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }

    let dx = (max[0] - min[0]).max(1e-10);
    let dy = (max[1] - min[1]).max(1e-10);
    let dz = (max[2] - min[2]).max(1e-10);
    let margin = (dx + dy + dz) * 10.0;
    let cx = (min[0] + max[0]) * 0.5;
    let cy = (min[1] + max[1]) * 0.5;
    let cz = (min[2] + max[2]) * 0.5;

    // Super-tetrahedron vertices (indices n..n+3)
    // Place them far enough to contain all points
    let super_pts = [
        [cx, cy + margin * 3.0, cz],
        [cx - margin * 3.0, cy - margin, cz - margin * 3.0],
        [cx + margin * 3.0, cy - margin, cz - margin * 3.0],
        [cx, cy - margin, cz + margin * 3.0],
    ];

    let mut all_pts: Vec<[f64; 3]> = pts.clone();
    all_pts.extend_from_slice(&super_pts);

    // Tetrahedra: [a, b, c, d] indices into all_pts
    let mut tets: Vec<[usize; 4]> = vec![[n, n + 1, n + 2, n + 3]];

    // Bowyer-Watson: insert each point
    for pi in 0..n {
        let p = all_pts[pi];

        // Find tetrahedra whose circumsphere contains the point
        let mut bad = vec![false; tets.len()];
        for (ti, tet) in tets.iter().enumerate() {
            if in_circumsphere(
                all_pts[tet[0]],
                all_pts[tet[1]],
                all_pts[tet[2]],
                all_pts[tet[3]],
                p,
            ) {
                bad[ti] = true;
            }
        }

        // Find boundary faces of the cavity
        // A face is on the boundary if it belongs to exactly one bad tetrahedron
        // For each bad tet, enumerate its 4 faces. A face is shared if another
        // bad tet also has a face with the same 3 vertices (unordered).
        let mut boundary_faces: Vec<[usize; 3]> = Vec::new();
        for (ti, tet) in tets.iter().enumerate() {
            if !bad[ti] {
                continue;
            }
            // Four faces, each ordered so that the normal points away from
            // the opposite vertex (for a positively-oriented tet)
            let opp = [
                (tet[0], [tet[1], tet[2], tet[3]]),
                (tet[1], [tet[0], tet[3], tet[2]]),
                (tet[2], [tet[0], tet[1], tet[3]]),
                (tet[3], [tet[0], tet[2], tet[1]]),
            ];
            for &(_opposite_vtx, face) in &opp {
                let mut sorted = [face[0], face[1], face[2]];
                sorted.sort();
                let shared = tets.iter().enumerate().any(|(tj, other)| {
                    if tj == ti || !bad[tj] {
                        return false;
                    }
                    has_face_unordered(other, &sorted)
                });
                if !shared {
                    boundary_faces.push(face);
                }
            }
        }

        // Remove bad tetrahedra and create new ones
        let mut new_tets: Vec<[usize; 4]> = tets
            .iter()
            .enumerate()
            .filter(|(ti, _)| !bad[*ti])
            .map(|(_, t)| *t)
            .collect();

        for face in &boundary_faces {
            new_tets.push([face[0], face[1], face[2], pi]);
        }

        tets = new_tets;
    }

    // Remove tetrahedra referencing super-tetrahedron vertices
    tets.retain(|tet| tet[0] < n && tet[1] < n && tet[2] < n && tet[3] < n);

    // Build output UnstructuredGrid
    let mut grid = UnstructuredGrid::new();
    for i in 0..n {
        grid.points.push(input.points.get(i));
    }

    for tet in &tets {
        grid.push_cell(
            CellType::Tetra,
            &[tet[0] as i64, tet[1] as i64, tet[2] as i64, tet[3] as i64],
        );
    }

    grid
}

/// Check if point p lies inside the circumsphere of tetrahedron (a, b, c, d).
///
/// Uses the determinant method: the point is inside if the determinant of
/// the 5x5 matrix (with the lifted paraboloid) is positive (assuming
/// consistent vertex orientation).
fn in_circumsphere(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3], p: [f64; 3]) -> bool {
    let ax = a[0] - p[0];
    let ay = a[1] - p[1];
    let az = a[2] - p[2];
    let bx = b[0] - p[0];
    let by = b[1] - p[1];
    let bz = b[2] - p[2];
    let cx = c[0] - p[0];
    let cy = c[1] - p[1];
    let cz = c[2] - p[2];
    let dx = d[0] - p[0];
    let dy = d[1] - p[1];
    let dz = d[2] - p[2];

    let a2 = ax * ax + ay * ay + az * az;
    let b2 = bx * bx + by * by + bz * bz;
    let c2 = cx * cx + cy * cy + cz * cz;
    let d2 = dx * dx + dy * dy + dz * dz;

    // 4x4 determinant (Cayley-Menger style)
    let det = ax * det3x3(by, bz, b2, cy, cz, c2, dy, dz, d2)
        - ay * det3x3(bx, bz, b2, cx, cz, c2, dx, dz, d2)
        + az * det3x3(bx, by, b2, cx, cy, c2, dx, dy, d2)
        - a2 * det3x3(bx, by, bz, cx, cy, cz, dx, dy, dz);

    // The sign depends on the orientation of (a,b,c,d). We check orientation
    // and flip the test accordingly.
    let orient = orient3d(a, b, c, d);
    if orient > 0.0 {
        det > 0.0
    } else {
        det < 0.0
    }
}

/// Orientation test for 4 points in 3D. Positive if d is above the plane (a,b,c)
/// with right-hand-rule normal.
fn orient3d(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> f64 {
    let ax = a[0] - d[0];
    let ay = a[1] - d[1];
    let az = a[2] - d[2];
    let bx = b[0] - d[0];
    let by = b[1] - d[1];
    let bz = b[2] - d[2];
    let cx = c[0] - d[0];
    let cy = c[1] - d[1];
    let cz = c[2] - d[2];

    ax * (by * cz - bz * cy) - ay * (bx * cz - bz * cx) + az * (bx * cy - by * cx)
}

fn det3x3(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, i: f64) -> f64 {
    a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
}

/// Check if a tetrahedron has a face with the same 3 vertices (unordered, pre-sorted).
fn has_face_unordered(tet: &[usize; 4], sorted_face: &[usize; 3]) -> bool {
    let faces = [
        [tet[1], tet[2], tet[3]],
        [tet[0], tet[2], tet[3]],
        [tet[0], tet[1], tet[3]],
        [tet[0], tet[1], tet[2]],
    ];
    for f in &faces {
        let mut s = [f[0], f[1], f[2]];
        s.sort();
        if s == *sorted_face {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSet;

    #[test]
    fn tetrahedralize_five_points() {
        // 4 corners of a tetrahedron + 1 interior point
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.points.push([0.5, 0.4, 0.3]); // interior point

        let result = delaunay_3d(&pd);
        assert_eq!(result.num_points(), 5);
        // 5 points in general position -> at least 2 tetrahedra
        assert!(
            result.num_cells() >= 2,
            "expected >= 2 tets, got {}",
            result.num_cells()
        );
        // All cells should be tetrahedra
        for i in 0..result.num_cells() {
            assert_eq!(result.cell_type(i), CellType::Tetra);
        }
    }

    #[test]
    fn tetrahedralize_cube_corners() {
        let mut pd = PolyData::new();
        // 8 corners of a unit cube
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    pd.points.push([x as f64, y as f64, z as f64]);
                }
            }
        }

        let result = delaunay_3d(&pd);
        assert_eq!(result.num_points(), 8);
        // A cube with 8 points -> 5 or 6 tetrahedra (depending on triangulation)
        assert!(
            result.num_cells() >= 5,
            "expected >= 5 tets, got {}",
            result.num_cells()
        );
        assert!(
            result.num_cells() <= 12,
            "expected <= 12 tets, got {}",
            result.num_cells()
        );
    }

    #[test]
    fn tetrahedralize_four_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);

        let result = delaunay_3d(&pd);
        assert_eq!(result.num_points(), 4);
        assert_eq!(result.num_cells(), 1); // exactly 1 tetrahedron
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        let result = delaunay_3d(&pd);
        assert_eq!(result.num_cells(), 0);
    }

    #[test]
    fn tetrahedralize_random_cloud() {
        // Deterministic "random" point cloud
        let mut pd = PolyData::new();
        let seeds: Vec<[f64; 3]> = vec![
            [0.1, 0.2, 0.3],
            [0.9, 0.1, 0.4],
            [0.5, 0.8, 0.2],
            [0.3, 0.5, 0.9],
            [0.7, 0.7, 0.7],
            [0.2, 0.9, 0.5],
            [0.8, 0.3, 0.8],
            [0.4, 0.4, 0.1],
            [0.6, 0.1, 0.6],
            [0.1, 0.6, 0.8],
        ];
        for p in &seeds {
            pd.points.push(*p);
        }

        let result = delaunay_3d(&pd);
        assert_eq!(result.num_points(), 10);
        assert!(
            result.num_cells() >= 5,
            "expected >= 5 tets, got {}",
            result.num_cells()
        );
    }
}
