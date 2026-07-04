use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Simple vertex clustering decimation.
///
/// Divides the bounding box of the mesh into a regular grid of `grid_size^3`
/// cells, selects a representative point for each occupied cell, and rebuilds
/// triangle faces. Degenerate faces (where two or more vertices map to the
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

    // Cell size in each dimension.
    let cell_size: [f64; 3] = [
        (max_pt[0] - min_pt[0]) / gs as f64,
        (max_pt[1] - min_pt[1]) / gs as f64,
        (max_pt[2] - min_pt[2]) / gs as f64,
    ];

    // Map each vertex to a grid cell.
    let mut vertex_to_cell: Vec<usize> = vec![0; n];
    let mut cell_points: HashMap<usize, Vec<usize>> = HashMap::new();

    for i in 0..n {
        let p = input.points.get(i);
        let ix = cluster_coord(p[0], min_pt[0], cell_size[0], gs);
        let iy = cluster_coord(p[1], min_pt[1], cell_size[1], gs);
        let iz = cluster_coord(p[2], min_pt[2], cell_size[2], gs);
        let cell_id: usize = iz * gs * gs + iy * gs + ix;
        vertex_to_cell[i] = cell_id;
        cell_points.entry(cell_id).or_default().push(i);
    }

    // Build new points: one per occupied cell.
    let mut cell_to_new_idx: HashMap<usize, usize> = HashMap::new();
    let mut new_points = Points::<f64>::new();

    let mut sorted_cells: Vec<usize> = cell_points.keys().copied().collect();
    sorted_cells.sort();

    for &cell_id in &sorted_cells {
        let points = &cell_points[&cell_id];
        let representative = points[points.len() / 2];
        new_points.push(input.points.get(representative));
        cell_to_new_idx.insert(cell_id, new_points.len() - 1);
    }

    // Rebuild triangles, skipping degenerate ones.
    let mut new_cells: Vec<[i64; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let mut new_ids = [
            cell_to_new_idx[&vertex_to_cell[cell[0] as usize]] as i64,
            cell_to_new_idx[&vertex_to_cell[cell[1] as usize]] as i64,
            cell_to_new_idx[&vertex_to_cell[cell[2] as usize]] as i64,
        ];
        if new_ids[0] != new_ids[1] && new_ids[0] != new_ids[2] && new_ids[1] != new_ids[2] {
            rotate_smallest_first(&mut new_ids);
            new_cells.push(new_ids);
        }
    }

    new_cells.sort();
    new_cells.dedup();

    let mut new_polys = CellArray::new();
    for cell in new_cells {
        new_polys.push_cell(&cell);
    }

    let mut pd = PolyData::new();
    pd.points = new_points;
    pd.polys = new_polys;
    pd
}

fn cluster_coord(value: f64, origin: f64, cell_size: f64, grid_size: usize) -> usize {
    if cell_size > 0.0 {
        (((value - origin) / cell_size).floor() as usize).min(grid_size - 1)
    } else {
        0
    }
}

fn rotate_smallest_first(ids: &mut [i64; 3]) {
    if ids[0] > ids[1] || ids[0] > ids[2] {
        ids.rotate_left(1);
        if ids[0] > ids[1] || ids[0] > ids[2] {
            ids.rotate_left(1);
        }
    }
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

    #[test]
    fn removes_duplicate_clustered_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([0.0, 10.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([10.1, 0.0, 0.0]);
        pd.points.push([0.1, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = decimate_vertex_cluster(&pd, 2);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn selects_middle_representative_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.2, 0.0, 0.0]);
        pd.points.push([0.4, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = decimate_vertex_cluster(&pd, 1);
        assert_eq!(result.points.get(0), [0.2, 0.0, 0.0]);
    }
}
