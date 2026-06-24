use crate::data::PolyData;
use std::collections::HashMap;

/// Comprehensive watertight check for a triangle mesh.
///
/// Returns a report with all issues found.
#[derive(Debug, Clone)]
pub struct WatertightReport {
    pub is_watertight: bool,
    pub num_boundary_edges: usize,
    pub num_non_manifold_edges: usize,
    pub num_inconsistent_edges: usize,
    pub num_degenerate_faces: usize,
    pub num_isolated_vertices: usize,
}

/// Check if a mesh is watertight (closed, manifold, consistent).
pub fn watertight_check(input: &PolyData) -> WatertightReport {
    let n = input.points.len();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut directed_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut used = vec![false; n];
    let mut degenerate = 0;

    for c in &cells {
        if c.len() < 3 {
            degenerate += 1;
            continue;
        }
        // Check degenerate (zero area)
        let v0 = input.points.get(c[0] as usize);
        let v1 = input.points.get(c[1] as usize);
        let v2 = input.points.get(c[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        if cx * cx + cy * cy + cz * cz < 1e-30 {
            degenerate += 1;
        }

        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            used[a as usize] = true;
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
            *directed_count.entry((a, b)).or_insert(0) += 1;
        }
    }

    let boundary = edge_count.values().filter(|&&c| c == 1).count();
    let non_manifold = edge_count.values().filter(|&&c| c > 2).count();
    let inconsistent = directed_count.values().filter(|&&c| c > 1).count();
    let isolated = used.iter().filter(|&&u| !u).count();

    let is_watertight = boundary == 0 && non_manifold == 0 && inconsistent == 0 && degenerate == 0;

    WatertightReport {
        is_watertight,
        num_boundary_edges: boundary,
        num_non_manifold_edges: non_manifold,
        num_inconsistent_edges: inconsistent,
        num_degenerate_faces: degenerate,
        num_isolated_vertices: isolated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_tetrahedron() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let r = watertight_check(&pd);
        assert!(r.is_watertight);
        assert_eq!(r.num_boundary_edges, 0);
    }

    #[test]
    fn open_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let r = watertight_check(&pd);
        assert!(!r.is_watertight);
        assert!(r.num_boundary_edges > 0);
    }

    #[test]
    fn non_manifold_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[0, 1, 4]);

        let r = watertight_check(&pd);
        assert!(!r.is_watertight);
        assert!(r.num_non_manifold_edges > 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let r = watertight_check(&pd);
        assert!(r.is_watertight); // vacuously true
    }
}
