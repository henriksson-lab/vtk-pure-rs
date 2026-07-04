//! Planar mesh cutting with cap generation.
//!
//! Cuts a mesh by a plane and optionally fills the cut with a cap polygon
//! to produce a closed cross-section.

use crate::data::{CellArray, Points, PolyData};
use crate::filters::mesh::clip_by_plane::clip_by_plane;

/// Cut a mesh by a plane and generate a cap polygon at the cut.
///
/// Returns (clipped_mesh, cap_polygon).
/// Keeps the positive side of the plane and inserts intersection vertices on cut polygons.
pub fn cut_with_cap(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> (PolyData, PolyData) {
    let clipped = clip_by_plane(mesh, origin, normal);

    // Find boundary edges of the clipped mesh that lie on the cut plane.
    let boundary_loops = find_boundary_loops_on_plane(&clipped, origin, normal, 0.01);

    let cap = if boundary_loops.iter().any(|boundary| boundary.len() >= 3) {
        build_cap_polygons(&boundary_loops, normal)
    } else {
        PolyData::new()
    };

    (clipped, cap)
}

/// Cut and merge: returns a single closed mesh with the cap included.
pub fn cut_with_cap_merged(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let (mut clipped, cap) = cut_with_cap(mesh, origin, normal);
    if cap.points.len() > 0 {
        let offset = clipped.points.len() as i64;
        for i in 0..cap.points.len() {
            clipped.points.push(cap.points.get(i));
        }
        for cell in cap.polys.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            clipped.polys.push_cell(&shifted);
        }
    }
    clipped
}

fn find_boundary_on_plane(
    mesh: &PolyData,
    origin: [f64; 3],
    normal: [f64; 3],
    tolerance: f64,
) -> Vec<[f64; 3]> {
    find_boundary_loops_on_plane(mesh, origin, normal, tolerance)
        .into_iter()
        .flatten()
        .collect()
}

fn find_boundary_loops_on_plane(
    mesh: &PolyData,
    origin: [f64; 3],
    normal: [f64; 3],
    tolerance: f64,
) -> Vec<Vec<[f64; 3]>> {
    let nlen = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    if nlen < 1e-15 {
        return Vec::new();
    }
    let n = [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen];

    // Find boundary edges
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], mesh.points.len()) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], mesh.points.len()) else {
                continue;
            };
            if a == b {
                continue;
            }
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }

    let mut plane_pts: Vec<[f64; 3]> = Vec::new();
    let mut plane_edges: Vec<(usize, usize)> = Vec::new();

    for (&(a, b), &count) in &edge_count {
        if count != 1 {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let da =
            (pa[0] - origin[0]) * n[0] + (pa[1] - origin[1]) * n[1] + (pa[2] - origin[2]) * n[2];
        let db =
            (pb[0] - origin[0]) * n[0] + (pb[1] - origin[1]) * n[1] + (pb[2] - origin[2]) * n[2];
        if da.abs() < tolerance && db.abs() < tolerance {
            let ia = unique_plane_point_id(&mut plane_pts, pa, tolerance);
            let ib = unique_plane_point_id(&mut plane_pts, pb, tolerance);
            if ia != ib {
                plane_edges.push((ia, ib));
            }
        }
    }

    if plane_pts.is_empty() {
        return Vec::new();
    }

    let mut adjacency = vec![Vec::<usize>::new(); plane_pts.len()];
    for (a, b) in plane_edges {
        adjacency[a].push(b);
        adjacency[b].push(a);
    }

    let mut visited = vec![false; plane_pts.len()];
    let mut loops = Vec::new();
    for seed in 0..plane_pts.len() {
        if visited[seed] {
            continue;
        }
        let mut stack = vec![seed];
        visited[seed] = true;
        let mut component = Vec::new();
        while let Some(id) = stack.pop() {
            component.push(plane_pts[id]);
            for &next in &adjacency[id] {
                if !visited[next] {
                    visited[next] = true;
                    stack.push(next);
                }
            }
        }
        sort_points_on_plane(&mut component, n);
        loops.push(component);
    }

    loops
}

fn sort_points_on_plane(points: &mut [[f64; 3]], n: [f64; 3]) {
    if points.len() < 3 {
        return;
    }
    let cx = points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64;
    let cy = points.iter().map(|p| p[1]).sum::<f64>() / points.len() as f64;
    let cz = points.iter().map(|p| p[2]).sum::<f64>() / points.len() as f64;

    let up = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = cross(n, up);
    let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    let u = [u[0] / ul, u[1] / ul, u[2] / ul];
    let v = cross(n, u);

    points.sort_by(|a, b| {
        let da = [a[0] - cx, a[1] - cy, a[2] - cz];
        let db = [b[0] - cx, b[1] - cy, b[2] - cz];
        let angle_a = (da[0] * v[0] + da[1] * v[1] + da[2] * v[2])
            .atan2(da[0] * u[0] + da[1] * u[1] + da[2] * u[2]);
        let angle_b = (db[0] * v[0] + db[1] * v[1] + db[2] * v[2])
            .atan2(db[0] * u[0] + db[1] * u[1] + db[2] * u[2]);
        angle_a
            .partial_cmp(&angle_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn valid_point_id(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn build_cap_polygon(pts: &[[f64; 3]], normal: [f64; 3]) -> PolyData {
    let mut points = Points::<f64>::new();
    for p in pts {
        points.push(*p);
    }
    let mut polys = CellArray::new();

    // Fan triangulation. The clipped mesh keeps the positive side of the plane,
    // so the cap closes the cut with outward winding opposite the plane normal.
    let n = pts.len();
    let reverse = cap_normal_dot(pts, normal) > 0.0;
    if reverse {
        for i in 1..n - 1 {
            polys.push_cell(&[0, (i + 1) as i64, i as i64]);
        }
    } else {
        for i in 1..n - 1 {
            polys.push_cell(&[0, i as i64, (i + 1) as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

fn build_cap_polygons(loops: &[Vec<[f64; 3]>], normal: [f64; 3]) -> PolyData {
    let mut cap = PolyData::new();
    for pts in loops {
        if pts.len() < 3 {
            continue;
        }
        let loop_cap = build_cap_polygon(pts, normal);
        let offset = cap.points.len() as i64;
        for i in 0..loop_cap.points.len() {
            cap.points.push(loop_cap.points.get(i));
        }
        for cell in loop_cap.polys.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            cap.polys.push_cell(&shifted);
        }
    }
    cap
}

fn cap_normal_dot(pts: &[[f64; 3]], normal: [f64; 3]) -> f64 {
    for i in 1..pts.len().saturating_sub(1) {
        let e1 = [
            pts[i][0] - pts[0][0],
            pts[i][1] - pts[0][1],
            pts[i][2] - pts[0][2],
        ];
        let e2 = [
            pts[i + 1][0] - pts[0][0],
            pts[i + 1][1] - pts[0][1],
            pts[i + 1][2] - pts[0][2],
        ];
        let cap_normal = cross(e1, e2);
        let len2 = cap_normal[0] * cap_normal[0]
            + cap_normal[1] * cap_normal[1]
            + cap_normal[2] * cap_normal[2];
        if len2 > 1e-24 {
            return cap_normal[0] * normal[0]
                + cap_normal[1] * normal[1]
                + cap_normal[2] * normal[2];
        }
    }
    0.0
}

fn unique_plane_point_id(points: &mut Vec<[f64; 3]>, point: [f64; 3], tolerance: f64) -> usize {
    let tol2 = tolerance * tolerance;
    if let Some((idx, _)) = points.iter().enumerate().find(|(_, p)| {
        let dx = p[0] - point[0];
        let dy = p[1] - point[1];
        let dz = p[2] - point[2];
        dx * dx + dy * dy + dz * dz <= tol2
    }) {
        idx
    } else {
        let idx = points.len();
        points.push(point);
        idx
    }
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

    #[test]
    fn cut_sphere() {
        let sphere = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let (clipped, _cap) = cut_with_cap(&sphere, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(clipped.polys.num_cells() > 0);
        // Cap may or may not have points depending on tolerance
    }

    #[test]
    fn merged() {
        let sphere = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let result = cut_with_cap_merged(&sphere, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn cap_faces_opposite_cut_normal_for_positive_half_space() {
        let mut mesh = PolyData::new();
        for p in [
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ] {
            mesh.points.push(p);
        }
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        mesh.polys.push_cell(&[4, 7, 6, 5]);
        mesh.polys.push_cell(&[0, 4, 5, 1]);
        mesh.polys.push_cell(&[1, 5, 6, 2]);
        mesh.polys.push_cell(&[2, 6, 7, 3]);
        mesh.polys.push_cell(&[3, 7, 4, 0]);

        let (_, cap) = cut_with_cap(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);

        assert!(cap.polys.num_cells() > 0);
        for cell in cap.polys.iter() {
            let a = cap.points.get(cell[0] as usize);
            let b = cap.points.get(cell[1] as usize);
            let c = cap.points.get(cell[2] as usize);
            let normal = cross(
                [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            );
            assert!(normal[2] < 0.0, "cap triangle normal was {:?}", normal);
        }
    }

    #[test]
    fn malformed_boundary_ids_are_skipped() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.polys.push_cell(&[0, -1, 1]);

        let boundary = find_boundary_on_plane(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 0.01);

        assert_eq!(boundary.len(), 2);
    }

    #[test]
    fn disjoint_cut_loops_get_separate_caps() {
        let mut mesh = PolyData::new();
        append_cube(&mut mesh, -3.0);
        append_cube(&mut mesh, 3.0);

        let (_, cap) = cut_with_cap(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);

        assert_eq!(cap.points.len(), 8);
        assert_eq!(cap.polys.num_cells(), 4);
    }

    fn append_cube(mesh: &mut PolyData, x_offset: f64) {
        let offset = mesh.points.len() as i64;
        for p in [
            [-1.0 + x_offset, -1.0, -1.0],
            [1.0 + x_offset, -1.0, -1.0],
            [1.0 + x_offset, 1.0, -1.0],
            [-1.0 + x_offset, 1.0, -1.0],
            [-1.0 + x_offset, -1.0, 1.0],
            [1.0 + x_offset, -1.0, 1.0],
            [1.0 + x_offset, 1.0, 1.0],
            [-1.0 + x_offset, 1.0, 1.0],
        ] {
            mesh.points.push(p);
        }
        for cell in [
            [0, 1, 2, 3],
            [4, 7, 6, 5],
            [0, 4, 5, 1],
            [1, 5, 6, 2],
            [2, 6, 7, 3],
            [3, 7, 4, 0],
        ] {
            let shifted: Vec<i64> = cell.iter().map(|id| id + offset).collect();
            mesh.polys.push_cell(&shifted);
        }
    }
}
