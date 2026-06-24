use crate::data::{CellArray, Points, PolyData};

/// Simple vertex clustering decimation.
///
/// Divides the bounding box of the mesh into a regular grid of `grid_size^3`
/// cells, merges all vertices in each cell to their average position, and
/// rebuilds faces. Degenerate faces (where two or more vertices map to the
/// same cluster) are discarded.
pub fn decimate_vertex_cluster(input: &PolyData, grid_size: usize) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 || grid_size == 0 {
        return PolyData::new();
    }

    let gs: usize = grid_size.max(1);

    // Compute bounding box.
    let first = input.points.get(0);
    let mut min_pt: [f64; 3] = first;
    let mut max_pt: [f64; 3] = first;
    for i in 1..n {
        let p = input.points.get(i);
        for d in 0..3 {
            if p[d] < min_pt[d] {
                min_pt[d] = p[d];
            }
            if p[d] > max_pt[d] {
                max_pt[d] = p[d];
            }
        }
    }

    // Cell size in each dimension (add small epsilon to avoid boundary issues).
    let eps: f64 = 1e-10;
    let cell_size: [f64; 3] = [
        (max_pt[0] - min_pt[0] + eps) / gs as f64,
        (max_pt[1] - min_pt[1] + eps) / gs as f64,
        (max_pt[2] - min_pt[2] + eps) / gs as f64,
    ];

    // Map each vertex to a grid cell.
    let mut vertex_to_cell: Vec<usize> = vec![0; n];
    // Accumulate positions per cell.
    let mut cell_sum: std::collections::HashMap<usize, [f64; 3]> = std::collections::HashMap::new();
    let mut cell_count: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for i in 0..n {
        let p = input.points.get(i);
        let ix: usize = if cell_size[0] > 0.0 {
            ((p[0] - min_pt[0]) / cell_size[0]).floor() as usize
        } else {
            0
        };
        let iy: usize = if cell_size[1] > 0.0 {
            ((p[1] - min_pt[1]) / cell_size[1]).floor() as usize
        } else {
            0
        };
        let iz: usize = if cell_size[2] > 0.0 {
            ((p[2] - min_pt[2]) / cell_size[2]).floor() as usize
        } else {
            0
        };
        let cell_id: usize = iz * gs * gs + iy * gs + ix;
        vertex_to_cell[i] = cell_id;

        let sum = cell_sum.entry(cell_id).or_insert([0.0, 0.0, 0.0]);
        for d in 0..3 {
            sum[d] += p[d];
        }
        *cell_count.entry(cell_id).or_insert(0) += 1;
    }

    // Build new points: one per occupied cell.
    let mut cell_to_new_idx: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    let mut new_points = Points::<f64>::new();

    let mut sorted_cells: Vec<usize> = cell_sum.keys().copied().collect();
    sorted_cells.sort();

    for &cell_id in &sorted_cells {
        let sum = cell_sum[&cell_id];
        let cnt: f64 = cell_count[&cell_id] as f64;
        new_points.push([sum[0] / cnt, sum[1] / cnt, sum[2] / cnt]);
        cell_to_new_idx.insert(cell_id, new_points.len() - 1);
    }

    // Rebuild faces, skipping degenerate ones.
    let mut new_polys = CellArray::new();
    for cell in input.polys.iter() {
        let mut new_ids: Vec<i64> = Vec::with_capacity(cell.len());
        let mut seen = std::collections::HashSet::new();
        let mut degenerate: bool = false;
        for &vid in cell.iter() {
            let cell_id = vertex_to_cell[vid as usize];
            let new_id = cell_to_new_idx[&cell_id];
            if !seen.insert(new_id) {
                degenerate = true;
                break;
            }
            new_ids.push(new_id as i64);
        }
        if !degenerate && new_ids.len() >= 3 {
            new_polys.push_cell(&new_ids);
        }
    }

    let mut pd = PolyData::new();
    pd.points = new_points;
    pd.polys = new_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn basic_decimation() {
        // 4 closely spaced vertices forming 2 triangles, grid_size=1 merges all.
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.0, 0.1, 0.0]);
        pd.points.push([0.1, 0.1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);

        let result = decimate_vertex_cluster(&pd, 1);
        // All vertices merge to one cell -> all faces degenerate -> no faces.
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn preserves_well_separated() {
        // Three widely separated vertices with grid_size large enough to keep them distinct.
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = decimate_vertex_cluster(&pd, 10);
        // All vertices in separate cells -> triangle preserved.
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = decimate_vertex_cluster(&pd, 5);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
