use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::BTreeSet;

/// Edge length statistics for a mesh.
#[derive(Debug, Clone)]
pub struct EdgeLengthStats {
    /// Minimum edge length.
    pub min: f64,
    /// Maximum edge length.
    pub max: f64,
    /// Mean edge length.
    pub mean: f64,
    /// Number of unique edges.
    pub count: usize,
}

/// Compute per-edge lengths and return statistics (min, max, mean).
///
/// Also returns a line-based PolyData containing all unique edges as line
/// segments with an "EdgeLength" cell data array.
pub fn edge_length_stats(input: &PolyData) -> (EdgeLengthStats, PolyData) {
    let edge_set = collect_unique_edges(input);

    let mut points = Points::new();
    let mut lines = CellArray::new();
    let mut lengths: Vec<f64> = Vec::new();

    let mut min_l: f64 = f64::MAX;
    let mut max_l: f64 = 0.0;
    let mut sum_l: f64 = 0.0;

    let mut pt_idx: usize = 0;
    for &(a, b) in &edge_set {
        let pa = input.points.get(a);
        let pb = input.points.get(b);
        let dx: f64 = pa[0] - pb[0];
        let dy: f64 = pa[1] - pb[1];
        let dz: f64 = pa[2] - pb[2];
        let d: f64 = (dx * dx + dy * dy + dz * dz).sqrt();

        min_l = min_l.min(d);
        max_l = max_l.max(d);
        sum_l += d;

        points.push(pa);
        points.push(pb);
        lines.push_cell(&[pt_idx as i64, (pt_idx + 1) as i64]);
        lengths.push(d);
        pt_idx += 2;
    }

    let count: usize = edge_set.len();
    let mean_l: f64 = if count > 0 { sum_l / count as f64 } else { 0.0 };

    if count == 0 {
        min_l = 0.0;
        max_l = 0.0;
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "EdgeLength",
            lengths,
            1,
        )));

    let stats = EdgeLengthStats {
        min: min_l,
        max: max_l,
        mean: mean_l,
        count,
    };

    (stats, pd)
}

fn collect_unique_edges(input: &PolyData) -> BTreeSet<(usize, usize)> {
    let mut edge_set: BTreeSet<(usize, usize)> = BTreeSet::new();

    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            insert_edge(&mut edge_set, input.points.len(), edge[0], edge[1]);
        }
    }

    for cell in input.polys.iter() {
        let n = cell.len();
        for i in 0..n {
            insert_edge(
                &mut edge_set,
                input.points.len(),
                cell[i],
                cell[(i + 1) % n],
            );
        }
    }

    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut edge_set, input.points.len(), tri[0], tri[1]);
            insert_edge(&mut edge_set, input.points.len(), tri[1], tri[2]);
            insert_edge(&mut edge_set, input.points.len(), tri[2], tri[0]);
        }
    }

    edge_set
}

fn insert_edge(edge_set: &mut BTreeSet<(usize, usize)>, num_points: usize, a: i64, b: i64) {
    if a == b || a < 0 || b < 0 {
        return;
    }
    let a = a as usize;
    let b = b as usize;
    if a >= num_points || b >= num_points {
        return;
    }
    edge_set.insert(if a < b { (a, b) } else { (b, a) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equilateral_triangle_stats() {
        let s: f64 = (3.0f64).sqrt() / 2.0;
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, s, 0.0]],
            vec![[0i64, 1, 2]],
        );
        let (stats, line_pd) = edge_length_stats(&pd);
        assert_eq!(stats.count, 3);
        assert!((stats.min - 1.0).abs() < 1e-10);
        assert!((stats.max - 1.0).abs() < 1e-10);
        assert!((stats.mean - 1.0).abs() < 1e-10);
        assert_eq!(line_pd.lines.num_cells(), 3);
        let arr = line_pd.cell_data().get_array("EdgeLength").unwrap();
        assert_eq!(arr.num_tuples(), 3);
    }

    #[test]
    fn right_triangle_stats() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![[0i64, 1, 2]],
        );
        let (stats, _) = edge_length_stats(&pd);
        assert_eq!(stats.count, 3);
        assert!((stats.min - 3.0).abs() < 1e-10);
        assert!((stats.max - 5.0).abs() < 1e-10);
        let expected_mean: f64 = (3.0 + 4.0 + 5.0) / 3.0;
        assert!((stats.mean - expected_mean).abs() < 1e-10);
    }

    #[test]
    fn shared_edge_counted_once() {
        // Two triangles sharing an edge: should have 5 unique edges, not 6
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0i64, 1, 2], [0, 3, 1]],
        );
        let (stats, line_pd) = edge_length_stats(&pd);
        assert_eq!(stats.count, 5);
        assert_eq!(line_pd.lines.num_cells(), 5);
    }

    #[test]
    fn line_cells_are_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let (stats, line_pd) = edge_length_stats(&pd);
        assert_eq!(stats.count, 2);
        assert!((stats.min - 3.0).abs() < 1e-10);
        assert!((stats.max - 4.0).abs() < 1e-10);
        assert_eq!(line_pd.lines.num_cells(), 2);
    }
}
