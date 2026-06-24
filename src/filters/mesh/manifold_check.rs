//! Manifold mesh validation and repair utilities.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Detailed manifold check results.
#[derive(Debug, Clone)]
pub struct ManifoldReport {
    pub is_manifold: bool,
    pub is_closed: bool,
    pub is_consistent: bool,
    pub boundary_edges: usize,
    pub non_manifold_edges: usize,
    pub non_manifold_vertices: usize,
    pub degenerate_faces: usize,
}

impl std::fmt::Display for ManifoldReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "manifold={} closed={} consistent={} boundary={} non_manifold_edges={} non_manifold_verts={} degenerate={}",
            self.is_manifold, self.is_closed, self.is_consistent,
            self.boundary_edges, self.non_manifold_edges, self.non_manifold_vertices, self.degenerate_faces)
    }
}

/// Perform comprehensive manifold check.
pub fn check_manifold(mesh: &PolyData) -> ManifoldReport {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let mut directed: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut degenerate = 0;

    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 3 {
            degenerate += 1;
            continue;
        }
        // Check for duplicate vertices
        let mut ids: Vec<usize> = cell.iter().map(|&p| p as usize).collect();
        let unique: std::collections::HashSet<usize> = ids.iter().cloned().collect();
        if unique.len() < 3 {
            degenerate += 1;
            continue;
        }

        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            *directed.entry((a, b)).or_insert(0) += 1;
        }
    }

    let boundary = ec.values().filter(|&&c| c == 1).count();
    let non_manifold_edges = ec.values().filter(|&&c| c > 2).count();

    // Non-manifold vertices: vertices where the one-ring is not a single fan
    let n = mesh.points.len();
    let mut vert_edge_count: Vec<usize> = vec![0; n];
    for (&(a, b), _) in &ec {
        vert_edge_count[a] += 1;
        vert_edge_count[b] += 1;
    }
    let mut vert_face_count: Vec<usize> = vec![0; n];
    for cell in mesh.polys.iter() {
        for &pid in cell {
            vert_face_count[pid as usize] += 1;
        }
    }
    let non_manifold_verts = (0..n)
        .filter(|&i| {
            if vert_edge_count[i] == 0 {
                return false;
            }
            // For manifold: edges = faces (interior) or edges = faces+1 (boundary)
            let diff = (vert_edge_count[i] as i64) - (vert_face_count[i] as i64);
            diff < 0 || diff > 1
        })
        .count();

    // Consistency: each undirected edge should have one (a,b) and one (b,a)
    let inconsistent = directed.values().any(|&c| c > 1);

    ManifoldReport {
        is_manifold: non_manifold_edges == 0 && non_manifold_verts == 0,
        is_closed: boundary == 0,
        is_consistent: !inconsistent,
        boundary_edges: boundary,
        non_manifold_edges,
        non_manifold_vertices: non_manifold_verts,
        degenerate_faces: degenerate,
    }
}

/// Mark non-manifold edges and vertices.
pub fn mark_non_manifold(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }

    let mut data = vec![0.0f64; n];
    for (&(a, b), &count) in &ec {
        if count > 2 {
            data[a] = 1.0;
            data[b] = 1.0;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NonManifold",
            data,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifold_tet() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let report = check_manifold(&mesh);
        assert!(report.is_manifold);
        assert!(report.is_closed);
        assert_eq!(report.non_manifold_edges, 0);
    }
    #[test]
    fn open_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let report = check_manifold(&mesh);
        assert!(!report.is_closed);
        assert_eq!(report.boundary_edges, 3);
    }
    #[test]
    fn display() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = format!("{}", check_manifold(&mesh));
        assert!(s.contains("manifold="));
    }
    #[test]
    fn mark() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = mark_non_manifold(&mesh);
        assert!(result.point_data().get_array("NonManifold").is_some());
    }
}
