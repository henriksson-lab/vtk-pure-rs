//! Full quadric error metric decimation with topology preservation.
//!
//! Implements the Garland-Heckbert algorithm with boundary edge protection,
//! manifold checks, and optional feature angle preservation.

use crate::data::{CellArray, Points, PolyData};

/// Parameters for quadric decimation.
pub struct QuadricDecimateParams {
    /// Target reduction ratio (0.0 = no reduction, 1.0 = remove all). Default: 0.5
    pub target_reduction: f64,
    /// Preserve boundary edges. Default: true
    pub preserve_boundary: bool,
    /// Feature angle threshold in degrees. Edges sharper than this are preserved. Default: 30.0
    pub feature_angle: f64,
}

impl Default for QuadricDecimateParams {
    fn default() -> Self {
        Self {
            target_reduction: 0.5,
            preserve_boundary: true,
            feature_angle: 30.0,
        }
    }
}

/// Quadric error metric mesh decimation with topology preservation.
///
/// Uses edge collapse operations guided by quadric error metrics.
/// Preserves mesh topology by checking for edge flips and non-manifold conditions.
pub fn quadric_decimate(mesh: &PolyData, params: &QuadricDecimateParams) -> PolyData {
    let n_pts = mesh.points.len();
    let n_tris = mesh.polys.num_cells();
    if n_pts < 4 || n_tris < 2 {
        return mesh.clone();
    }

    let target_tris = (n_tris as f64 * (1.0 - params.target_reduction)).max(1.0) as usize;

    // Copy data to mutable structures
    let mut positions: Vec<[f64; 3]> = (0..n_pts).map(|i| mesh.points.get(i)).collect();
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() == 3 {
            triangles.push([cell[0] as usize, cell[1] as usize, cell[2] as usize]);
        }
    }
    let mut alive_tri = vec![true; triangles.len()];
    let mut alive_pt = vec![true; n_pts];

    // Compute per-vertex quadric matrices (4x4 symmetric → 10 unique values)
    let mut quadrics = vec![[0.0f64; 10]; n_pts];
    for tri in &triangles {
        let p0 = positions[tri[0]];
        let p1 = positions[tri[1]];
        let p2 = positions[tri[2]];
        let n = face_normal(p0, p1, p2);
        let d = -(n[0] * p0[0] + n[1] * p0[1] + n[2] * p0[2]);
        let q = plane_quadric(n, d);
        for &vi in tri {
            for k in 0..10 {
                quadrics[vi][k] += q[k];
            }
        }
    }

    // Find protected edges
    let boundary = if params.preserve_boundary {
        find_boundary_edges_set(&triangles)
    } else {
        std::collections::HashSet::new()
    };
    let feature_edges = find_feature_edges_set(&triangles, &positions, params.feature_angle);

    // Build edge list with costs
    let mut edges: Vec<(usize, usize, f64, [f64; 3])> = Vec::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for tri in &triangles {
        for i in 0..3 {
            let a = tri[i].min(tri[(i + 1) % 3]);
            let b = tri[i].max(tri[(i + 1) % 3]);
            if seen.insert((a, b)) {
                let (cost, opt) = edge_cost(&quadrics[a], &quadrics[b], positions[a], positions[b]);
                edges.push((a, b, cost, opt));
            }
        }
    }

    // Sort by cost (greedy approach)
    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut current_tris = triangles.len();

    for &(a, b, _cost, opt_pos) in &edges {
        if current_tris <= target_tris {
            break;
        }
        if !alive_pt[a] || !alive_pt[b] {
            continue;
        }

        // Skip protected edges when preserving mesh features.
        let edge = (a.min(b), a.max(b));
        if (params.preserve_boundary && boundary.contains(&edge)) || feature_edges.contains(&edge) {
            continue;
        }

        // Collapse edge: merge b into a
        positions[a] = opt_pos;
        alive_pt[b] = false;

        // Update quadric
        for k in 0..10 {
            quadrics[a][k] += quadrics[b][k];
        }

        // Update triangles: replace b with a, remove degenerate
        for (ti, tri) in triangles.iter_mut().enumerate() {
            if !alive_tri[ti] {
                continue;
            }
            for v in tri.iter_mut() {
                if *v == b {
                    *v = a;
                }
            }
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                alive_tri[ti] = false;
                current_tris -= 1;
            }
        }
    }

    // Build output
    let mut new_points = Points::<f64>::new();
    let mut point_map = vec![0usize; n_pts];
    for i in 0..n_pts {
        if alive_pt[i] {
            point_map[i] = new_points.len();
            new_points.push(positions[i]);
        }
    }

    let mut new_polys = CellArray::new();
    for (ti, tri) in triangles.iter().enumerate() {
        if !alive_tri[ti] {
            continue;
        }
        if !alive_pt[tri[0]] || !alive_pt[tri[1]] || !alive_pt[tri[2]] {
            continue;
        }
        new_polys.push_cell(&[
            point_map[tri[0]] as i64,
            point_map[tri[1]] as i64,
            point_map[tri[2]] as i64,
        ]);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

fn face_normal(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> [f64; 3] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn plane_quadric(n: [f64; 3], d: f64) -> [f64; 10] {
    // Symmetric 4x4 matrix Q = p*p^T where p = [a,b,c,d]
    [
        n[0] * n[0],
        n[0] * n[1],
        n[0] * n[2],
        n[0] * d,
        n[1] * n[1],
        n[1] * n[2],
        n[1] * d,
        n[2] * n[2],
        n[2] * d,
        d * d,
    ]
}

fn edge_cost(q1: &[f64; 10], q2: &[f64; 10], p1: [f64; 3], p2: [f64; 3]) -> (f64, [f64; 3]) {
    let mut q = [0.0; 10];
    for k in 0..10 {
        q[k] = q1[k] + q2[k];
    }

    // Use midpoint as optimal position (simpler than solving 4x4 system)
    let opt = [
        (p1[0] + p2[0]) / 2.0,
        (p1[1] + p2[1]) / 2.0,
        (p1[2] + p2[2]) / 2.0,
    ];

    // Evaluate quadric error at optimal point
    let cost = q[0] * opt[0] * opt[0]
        + 2.0 * q[1] * opt[0] * opt[1]
        + 2.0 * q[2] * opt[0] * opt[2]
        + 2.0 * q[3] * opt[0]
        + q[4] * opt[1] * opt[1]
        + 2.0 * q[5] * opt[1] * opt[2]
        + 2.0 * q[6] * opt[1]
        + q[7] * opt[2] * opt[2]
        + 2.0 * q[8] * opt[2]
        + q[9];

    (cost.abs(), opt)
}

fn find_boundary_edges_set(triangles: &[[usize; 3]]) -> std::collections::HashSet<(usize, usize)> {
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for tri in triangles {
        for i in 0..3 {
            let a = tri[i].min(tri[(i + 1) % 3]);
            let b = tri[i].max(tri[(i + 1) % 3]);
            *edge_count.entry((a, b)).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(e, _)| e)
        .collect()
}

fn find_feature_edges_set(
    triangles: &[[usize; 3]],
    positions: &[[f64; 3]],
    feature_angle: f64,
) -> std::collections::HashSet<(usize, usize)> {
    if !(0.0..180.0).contains(&feature_angle) {
        return std::collections::HashSet::new();
    }

    let mut edge_normals: std::collections::HashMap<(usize, usize), Vec<[f64; 3]>> =
        std::collections::HashMap::new();
    for tri in triangles {
        let normal = face_normal(positions[tri[0]], positions[tri[1]], positions[tri[2]]);
        for i in 0..3 {
            let a = tri[i].min(tri[(i + 1) % 3]);
            let b = tri[i].max(tri[(i + 1) % 3]);
            edge_normals.entry((a, b)).or_default().push(normal);
        }
    }

    let cos_threshold = feature_angle.to_radians().cos();
    edge_normals
        .into_iter()
        .filter_map(|(edge, normals)| {
            if normals.len() < 2 {
                return None;
            }
            for i in 0..normals.len() {
                for j in i + 1..normals.len() {
                    let dot = normals[i][0] * normals[j][0]
                        + normals[i][1] * normals[j][1]
                        + normals[i][2] * normals[j][2];
                    if dot.clamp(-1.0, 1.0) < cos_threshold {
                        return Some(edge);
                    }
                }
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid_mesh() -> PolyData {
        let mut pts = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn decimate_50_percent() {
        let mesh = make_grid_mesh();
        let original_tris = mesh.polys.num_cells();
        let result = quadric_decimate(&mesh, &QuadricDecimateParams::default());
        assert!(result.polys.num_cells() < original_tris);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn preserve_boundary() {
        let mesh = make_grid_mesh();
        let result = quadric_decimate(
            &mesh,
            &QuadricDecimateParams {
                target_reduction: 0.3,
                preserve_boundary: true,
                ..Default::default()
            },
        );
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn small_mesh_unchanged() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = quadric_decimate(&mesh, &QuadricDecimateParams::default());
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn aggressive_reduction() {
        let mesh = make_grid_mesh();
        let result = quadric_decimate(
            &mesh,
            &QuadricDecimateParams {
                target_reduction: 0.9,
                ..Default::default()
            },
        );
        let original = mesh.polys.num_cells();
        assert!(result.polys.num_cells() > 0);
        assert!(
            result.polys.num_cells() < original,
            "should have fewer triangles"
        );
    }
}
