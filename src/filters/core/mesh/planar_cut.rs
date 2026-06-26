//! Planar mesh cutting with cap generation.
//!
//! Cuts a mesh by a plane and optionally fills the cut with a cap polygon
//! to produce a closed cross-section.

use crate::data::{CellArray, Points, PolyData};
use crate::filters::core::mesh::clip_by_plane::clip_by_plane;

/// Cut a mesh by a plane and generate a cap polygon at the cut.
///
/// Returns (clipped_mesh, cap_polygon).
/// Keeps the positive side of the plane and inserts intersection vertices on cut polygons.
pub fn cut_with_cap(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> (PolyData, PolyData) {
    let clipped = clip_by_plane(mesh, origin, normal);

    // Find boundary edges of the clipped mesh that lie on the cut plane
    let boundary = find_boundary_on_plane(&clipped, origin, normal, 0.01);

    let cap = if boundary.len() >= 3 {
        build_cap_polygon(&boundary, normal)
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
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }

    let mut plane_pts: Vec<[f64; 3]> = Vec::new();
    let mut seen: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (&(a, b), &count) in &edge_count {
        if count != 1 {
            continue;
        }
        for &vid in &[a, b] {
            if seen.contains(&vid) {
                continue;
            }
            let p = mesh.points.get(vid);
            let dist =
                (p[0] - origin[0]) * n[0] + (p[1] - origin[1]) * n[1] + (p[2] - origin[2]) * n[2];
            if dist.abs() < tolerance {
                plane_pts.push(p);
                seen.insert(vid);
            }
        }
    }

    // Sort by angle around centroid for proper polygon winding
    if plane_pts.len() >= 3 {
        let cx = plane_pts.iter().map(|p| p[0]).sum::<f64>() / plane_pts.len() as f64;
        let cy = plane_pts.iter().map(|p| p[1]).sum::<f64>() / plane_pts.len() as f64;
        let cz = plane_pts.iter().map(|p| p[2]).sum::<f64>() / plane_pts.len() as f64;

        let up = if n[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = cross(n, up);
        let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let u = [u[0] / ul, u[1] / ul, u[2] / ul];
        let v = cross(n, u);

        plane_pts.sort_by(|a, b| {
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

    plane_pts
}

fn build_cap_polygon(pts: &[[f64; 3]], _normal: [f64; 3]) -> PolyData {
    let mut points = Points::<f64>::new();
    for p in pts {
        points.push(*p);
    }
    let mut polys = CellArray::new();

    // Fan triangulation
    let n = pts.len();
    for i in 1..n - 1 {
        polys.push_cell(&[0, i as i64, (i + 1) as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
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
        let (clipped, cap) = cut_with_cap(&sphere, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
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
}
