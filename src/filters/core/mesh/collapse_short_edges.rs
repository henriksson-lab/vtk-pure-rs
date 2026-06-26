use crate::data::{CellArray, Points, PolyData};

/// Collapse edges shorter than a threshold using k-d tree for efficiency.
///
/// Unlike `collapse_edges`, this uses a k-d tree to find merge candidates
/// efficiently. Produces a cleaner mesh with better degenerate handling.
pub fn collapse_short_edges_kdtree(input: &PolyData, min_length: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || !min_length.is_finite() || min_length <= 0.0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = crate::data::KdTree::build(&pts);
    let d2 = min_length * min_length;

    // Build merge map
    let mut remap = vec![usize::MAX; n];
    let mut out_pts = Points::<f64>::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let idx = out_pts.len();
        let nbrs = tree.find_within_radius(pts[i], min_length);

        // Compute centroid of cluster
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let mut cnt = 0;
        for &(j, jd2) in &nbrs {
            if remap[j] == usize::MAX && jd2 <= d2 {
                cx += pts[j][0];
                cy += pts[j][1];
                cz += pts[j][2];
                cnt += 1;
                remap[j] = idx;
            }
        }
        if cnt > 0 {
            out_pts.push([cx / cnt as f64, cy / cnt as f64, cz / cnt as f64]);
        }
    }

    let remap_cells = |cells: &CellArray, min_unique: usize| {
        let mut out_cells = CellArray::new();
        for cell in cells.iter() {
            if cell.iter().any(|&id| id < 0 || id as usize >= n) {
                continue;
            }
            let mapped: Vec<i64> = cell.iter().map(|&id| remap[id as usize] as i64).collect();
            let mut unique = Vec::new();
            for id in mapped {
                if !unique.contains(&id) {
                    unique.push(id);
                }
            }
            if unique.len() >= min_unique {
                out_cells.push_cell(&unique);
            }
        }
        out_cells
    };

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = remap_cells(&input.verts, 1);
    pd.lines = remap_cells(&input.lines, 2);
    pd.polys = remap_cells(&input.polys, 3);
    pd.strips = remap_cells(&input.strips, 3);
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_close_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[1, 2, 3]);

        let result = collapse_short_edges_kdtree(&pd, 0.01);
        assert!(result.points.len() < 4);
    }

    #[test]
    fn preserves_well_spaced() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = collapse_short_edges_kdtree(&pd, 0.001);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(collapse_short_edges_kdtree(&pd, 0.1).points.len(), 0);
    }

    #[test]
    fn preserves_vertex_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.verts.push_cell(&[0]);
        pd.verts.push_cell(&[1]);

        let result = collapse_short_edges_kdtree(&pd, 0.1);
        assert_eq!(result.verts.num_cells(), 2);
    }

    #[test]
    fn removes_collapsed_vertices_from_output_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = collapse_short_edges_kdtree(&pd, 0.1);
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|cell| cell.to_vec()).collect();
        assert!(cells.iter().all(|cell| {
            let mut unique = cell.clone();
            unique.sort();
            unique.dedup();
            unique.len() == cell.len()
        }));
    }

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = collapse_short_edges_kdtree(&pd, 0.1);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
