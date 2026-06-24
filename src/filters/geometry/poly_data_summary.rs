use crate::data::{DataSet, PolyData};

/// Summary statistics for a PolyData mesh.
#[derive(Debug, Clone)]
pub struct PolyDataSummary {
    pub num_points: usize,
    pub num_polys: usize,
    pub num_lines: usize,
    pub num_verts: usize,
    pub num_strips: usize,
    pub num_point_arrays: usize,
    pub num_cell_arrays: usize,
    pub bounds: [f64; 6],
    pub memory_estimate_bytes: usize,
}

/// Compute a summary of a PolyData mesh.
pub fn poly_data_summary(input: &PolyData) -> PolyDataSummary {
    let bb = input.bounds();
    let n_pts = input.points.len();

    // Rough memory estimate
    let pts_bytes = n_pts * 3 * 8; // 3 * f64
    let cells_bytes = (input.polys.num_cells()
        + input.lines.num_cells()
        + input.verts.num_cells()
        + input.strips.num_cells())
        * 4
        * 8; // rough

    PolyDataSummary {
        num_points: n_pts,
        num_polys: input.polys.num_cells(),
        num_lines: input.lines.num_cells(),
        num_verts: input.verts.num_cells(),
        num_strips: input.strips.num_cells(),
        num_point_arrays: input.point_data().num_arrays(),
        num_cell_arrays: input.cell_data().num_arrays(),
        bounds: [bb.x_min, bb.x_max, bb.y_min, bb.y_max, bb.z_min, bb.z_max],
        memory_estimate_bytes: pts_bytes + cells_bytes,
    }
}

/// Check if a mesh is a valid triangle mesh (all polys are triangles).
pub fn is_triangle_mesh(input: &PolyData) -> bool {
    input.polys.iter().all(|cell| cell.len() == 3)
}

/// Check if a mesh has consistent topology (no non-manifold edges).
pub fn is_manifold(input: &PolyData) -> bool {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    // Manifold: every edge shared by at most 2 faces
    edge_count.values().all(|&c| c <= 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let s = poly_data_summary(&pd);
        assert_eq!(s.num_points, 3);
        assert_eq!(s.num_polys, 1);
        assert_eq!(s.num_lines, 0);
    }

    #[test]
    fn is_triangle_mesh_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        assert!(is_triangle_mesh(&pd));

        // Add a quad -> no longer triangle mesh
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 3, 2]);
        assert!(!is_triangle_mesh(&pd));
    }

    #[test]
    fn manifold_check() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        assert!(is_manifold(&pd)); // edge 0-1 shared by 2 faces = ok
    }

    #[test]
    fn non_manifold() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, -1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[0, 1, 4]); // edge 0-1 now shared by 3 faces
        assert!(!is_manifold(&pd));
    }
}
