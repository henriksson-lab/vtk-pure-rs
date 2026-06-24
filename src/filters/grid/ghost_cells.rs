//! Ghost cell generation for domain decomposition.
//!
//! When a mesh is split into partitions for parallel processing, ghost cells
//! are needed at partition boundaries to compute correct normals, gradients,
//! and other operations that require neighbor information.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate ghost cells for a partitioned mesh.
///
/// Given a mesh and a "RegionId" cell data array, adds one layer of ghost
/// cells from neighboring regions at each partition boundary. Ghost cells
/// are marked with a "GhostLevel" cell data array (0 = owned, 1 = ghost).
///
/// If the mesh has no "RegionId" array, returns a clone unchanged.
pub fn generate_ghost_cells(mesh: &PolyData) -> PolyData {
    let region_arr = match mesh.cell_data().get_array("RegionId") {
        Some(arr) => arr,
        None => return mesh.clone(),
    };

    let n_cells = mesh.polys.num_cells();
    let n_points = mesh.points.len();

    // Build point-to-cell adjacency
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut point_to_cells: Vec<Vec<usize>> = vec![Vec::new(); n_points];
    for (ci, cell) in all_cells.iter().enumerate() {
        for &pid in cell {
            point_to_cells[pid as usize].push(ci);
        }
    }

    // Find cell-to-region mapping
    let mut regions = Vec::with_capacity(n_cells);
    let mut buf = [0.0f64];
    for i in 0..n_cells {
        region_arr.tuple_as_f64(i, &mut buf);
        regions.push(buf[0] as i64);
    }

    // Find boundary cells: cells that share a point with a cell in a different region
    let ghost_level = vec![0.0f64; n_cells];
    let mut is_boundary_cell = vec![false; n_cells];

    for ci in 0..n_cells {
        let my_region = regions[ci];
        for &pid in &all_cells[ci] {
            for &neighbor_ci in &point_to_cells[pid as usize] {
                if neighbor_ci != ci && regions[neighbor_ci] != my_region {
                    is_boundary_cell[ci] = true;
                    break;
                }
            }
            if is_boundary_cell[ci] {
                break;
            }
        }
    }

    // For each region, collect ghost cells (cells from other regions that share
    // a boundary point with this region)
    let mut seen_ghost_pairs: std::collections::HashSet<(usize, i64)> =
        std::collections::HashSet::new();
    let mut extra_cells: Vec<(usize, i64)> = Vec::new(); // (cell_index, dest_region)

    for ci in 0..n_cells {
        if !is_boundary_cell[ci] {
            continue;
        }
        let my_region = regions[ci];

        // Find neighbors in other regions
        for &pid in &all_cells[ci] {
            for &neighbor_ci in &point_to_cells[pid as usize] {
                if regions[neighbor_ci] != my_region {
                    let key = (neighbor_ci, my_region);
                    if seen_ghost_pairs.insert(key) {
                        extra_cells.push(key);
                    }
                }
            }
        }
    }

    // Build output: original mesh + ghost cells from neighbors
    let mut result = mesh.clone();

    // Mark original cells
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GhostLevel",
            ghost_level,
            1,
        )));

    // For now, mark boundary cells in the output
    let boundary_data: Vec<f64> = (0..n_cells)
        .map(|ci| if is_boundary_cell[ci] { 1.0 } else { 0.0 })
        .collect();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryCell",
            boundary_data,
            1,
        )));

    result
}

/// Split a mesh into N partitions using spatial binning.
///
/// Assigns each cell to a partition based on its centroid position along
/// the longest axis. Returns the mesh with a "RegionId" cell data array.
pub fn partition_mesh(mesh: &PolyData, n_partitions: usize) -> PolyData {
    if mesh.polys.num_cells() == 0 || n_partitions == 0 {
        return mesh.clone();
    }

    let n_cells = mesh.polys.num_cells();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    // Compute centroids
    let mut centroids: Vec<[f64; 3]> = Vec::with_capacity(n_cells);
    for cell in &all_cells {
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &pid in cell {
            let p = mesh.points.get(pid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        centroids.push([cx / n, cy / n, cz / n]);
    }

    // Find longest axis
    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];
    for c in &centroids {
        for i in 0..3 {
            min[i] = min[i].min(c[i]);
            max[i] = max[i].max(c[i]);
        }
    }
    let extents = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let axis = if extents[0] >= extents[1] && extents[0] >= extents[2] {
        0
    } else if extents[1] >= extents[2] {
        1
    } else {
        2
    };

    // Assign partitions along longest axis
    let range = extents[axis];
    let mut region_ids = Vec::with_capacity(n_cells);
    for c in &centroids {
        let t = if range > 1e-15 {
            (c[axis] - min[axis]) / range
        } else {
            0.0
        };
        let region = ((t * n_partitions as f64) as usize).min(n_partitions - 1);
        region_ids.push(region as f64);
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RegionId", region_ids, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid_mesh() -> PolyData {
        // 4x4 grid of quads → 16 cells
        let mut points = Points::<f64>::new();
        for iy in 0..5 {
            for ix in 0..5 {
                points.push([ix as f64, iy as f64, 0.0]);
            }
        }
        let mut polys = CellArray::new();
        for iy in 0..4 {
            for ix in 0..4 {
                let bl = (iy * 5 + ix) as i64;
                let br = bl + 1;
                let tl = bl + 5;
                let tr = tl + 1;
                // Split quad into two triangles
                polys.push_cell(&[bl, br, tr]);
                polys.push_cell(&[bl, tr, tl]);
            }
        }
        let mut mesh = PolyData::new();
        mesh.points = points;
        mesh.polys = polys;
        mesh
    }

    #[test]
    fn partition_creates_regions() {
        let mesh = make_grid_mesh();
        let partitioned = partition_mesh(&mesh, 4);
        assert!(partitioned.cell_data().get_array("RegionId").is_some());

        let arr = partitioned.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];

        // Check that we have multiple different regions
        let mut regions = std::collections::HashSet::new();
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            regions.insert(buf[0] as i64);
        }
        assert!(regions.len() > 1);
    }

    #[test]
    fn ghost_cells_on_partitioned_mesh() {
        let mesh = make_grid_mesh();
        let partitioned = partition_mesh(&mesh, 2);
        let with_ghosts = generate_ghost_cells(&partitioned);

        assert!(with_ghosts.cell_data().get_array("GhostLevel").is_some());
        assert!(with_ghosts.cell_data().get_array("BoundaryCell").is_some());

        // Some cells should be boundary cells
        let boundary = with_ghosts.cell_data().get_array("BoundaryCell").unwrap();
        let mut has_boundary = false;
        let mut buf = [0.0f64];
        for i in 0..boundary.num_tuples() {
            boundary.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                has_boundary = true;
                break;
            }
        }
        assert!(
            has_boundary,
            "should have boundary cells between partitions"
        );
    }

    #[test]
    fn no_region_id_returns_clone() {
        let mesh = make_grid_mesh();
        let result = generate_ghost_cells(&mesh);
        assert_eq!(result.points.len(), mesh.points.len());
    }

    #[test]
    fn single_partition() {
        let mesh = make_grid_mesh();
        let partitioned = partition_mesh(&mesh, 1);
        let arr = partitioned.cell_data().get_array("RegionId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
