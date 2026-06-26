use crate::data::{DataSet, PolyData};
use std::collections::HashSet;

/// Comprehensive mesh statistics summary as a printable string.
pub fn mesh_stats_report(input: &PolyData) -> String {
    let n_pts = input.points.len();
    let n_polys = input.polys.num_cells();
    let n_lines = input.lines.num_cells();
    let n_verts = input.verts.num_cells();
    let n_strips = input.strips.num_cells();
    let n_pt_arrays = input.point_data().num_arrays();
    let n_cell_arrays = input.cell_data().num_arrays();
    let bb = input.bounds();

    let edge_set = unique_edges(input);

    format!(
        "Mesh Statistics:\n\
         Points: {}\n\
         Polygons: {}\n\
         Lines: {}\n\
         Vertices: {}\n\
         Strips: {}\n\
         Edges: {}\n\
         Point arrays: {}\n\
         Cell arrays: {}\n\
         Bounds: [{:.3}, {:.3}] x [{:.3}, {:.3}] x [{:.3}, {:.3}]\n\
         Diagonal: {:.3}",
        n_pts,
        n_polys,
        n_lines,
        n_verts,
        n_strips,
        edge_set.len(),
        n_pt_arrays,
        n_cell_arrays,
        bb.x_min,
        bb.x_max,
        bb.y_min,
        bb.y_max,
        bb.z_min,
        bb.z_max,
        bb.diagonal_length()
    )
}

/// Check if mesh is empty (no points or cells).
pub fn is_empty(input: &PolyData) -> bool {
    input.points.len() == 0
}

/// Check if mesh has only triangles.
pub fn is_all_triangles(input: &PolyData) -> bool {
    input.polys.iter().all(|c| c.len() == 3)
}

/// Check if mesh has only quads.
pub fn is_all_quads(input: &PolyData) -> bool {
    input.polys.iter().all(|c| c.len() == 4)
}

fn unique_edges(input: &PolyData) -> HashSet<(i64, i64)> {
    let mut edges = HashSet::new();
    let num_pts = input.points.len() as i64;

    for cell in input.lines.iter() {
        for pair in cell.windows(2) {
            insert_edge(&mut edges, pair[0], pair[1], num_pts);
        }
    }

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(&mut edges, cell[i], cell[(i + 1) % cell.len()], num_pts);
        }
    }

    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for tri in strip.windows(3) {
            insert_edge(&mut edges, tri[0], tri[1], num_pts);
            insert_edge(&mut edges, tri[1], tri[2], num_pts);
            insert_edge(&mut edges, tri[2], tri[0], num_pts);
        }
    }

    edges
}

fn insert_edge(edges: &mut HashSet<(i64, i64)>, a: i64, b: i64, num_pts: i64) {
    if a < 0 || b < 0 || a >= num_pts || b >= num_pts || a == b {
        return;
    }
    edges.insert(if a < b { (a, b) } else { (b, a) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let report = mesh_stats_report(&pd);
        assert!(report.contains("Points: 3"));
        assert!(report.contains("Polygons: 1"));
    }

    #[test]
    fn report_counts_polyline_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let report = mesh_stats_report(&pd);
        assert!(report.contains("Lines: 1"));
        assert!(report.contains("Edges: 2"));
    }

    #[test]
    fn is_empty_test() {
        assert!(is_empty(&PolyData::new()));
        let mut pd = PolyData::new();
        pd.points.push([0.0; 3]);
        assert!(!is_empty(&pd));
    }

    #[test]
    fn triangle_check() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        assert!(is_all_triangles(&pd));
        assert!(!is_all_quads(&pd));
    }

    #[test]
    fn quad_check() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        assert!(!is_all_triangles(&pd));
        assert!(is_all_quads(&pd));
    }
}
