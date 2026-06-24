//! Depth-sorted polygon layers for order-independent transparency.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Split a mesh into depth-peeled layers by sorting faces front-to-back
/// relative to a view direction.
pub fn depth_peel_layers(
    mesh: &PolyData,
    view_direction: [f64; 3],
    num_layers: usize,
) -> Vec<PolyData> {
    if mesh.polys.num_cells() == 0 || num_layers == 0 {
        return Vec::new();
    }

    let cell_depths = compute_cell_depths(mesh, view_direction);

    let cells_per_layer = (cell_depths.len() + num_layers - 1) / num_layers;

    let mut layers = Vec::new();
    for chunk in cell_depths.chunks(cells_per_layer) {
        let layer = extract_cells_fast(mesh, chunk);
        if layer.polys.num_cells() > 0 {
            layers.push(layer);
        }
    }

    layers
}

/// Sort all faces of a mesh by depth for painter's algorithm rendering.
///
/// Returns a new PolyData with faces reordered front-to-back relative to
/// the view direction, with a "Depth" cell data array.
pub fn depth_sort_mesh(mesh: &PolyData, view_direction: [f64; 3]) -> PolyData {
    let nc = mesh.polys.num_cells();
    if nc == 0 {
        return mesh.clone();
    }

    let cell_depths = compute_cell_depths(mesh, view_direction);

    // Rebuild PolyData with sorted cell order using raw offsets/connectivity
    let offsets = mesh.polys.offsets();
    let conn = mesh.polys.connectivity();

    // Pre-count total connectivity size (same as input)
    let total_conn = conn.len();
    let mut new_conn = Vec::with_capacity(total_conn);
    let mut new_off = Vec::with_capacity(nc + 1);
    new_off.push(0i64);
    let mut depth_arr = Vec::with_capacity(nc);

    // Point remapping: use a flat Vec instead of HashMap
    let np = mesh.points.len();
    let src_pts = mesh.points.as_flat_slice();
    let mut pt_map: Vec<i64> = vec![-1; np];
    let mut pts_flat: Vec<f64> = Vec::with_capacity(np * 3);

    for &(ci, depth) in &cell_depths {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        for idx in start..end {
            let old_id = conn[idx] as usize;
            if pt_map[old_id] < 0 {
                pt_map[old_id] = (pts_flat.len() / 3) as i64;
                let b = old_id * 3;
                pts_flat.extend_from_slice(&src_pts[b..b + 3]);
            }
            new_conn.push(pt_map[old_id]);
        }
        new_off.push(new_conn.len() as i64);
        depth_arr.push(depth);
    }

    let mut result = PolyData::new();
    result.points = Points::from_flat_vec(pts_flat);
    result.polys = CellArray::from_raw(new_off, new_conn);
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Depth", depth_arr, 1,
        )));
    result
}

fn compute_cell_depths(mesh: &PolyData, view_direction: [f64; 3]) -> Vec<(usize, f64)> {
    let nc = mesh.polys.num_cells();
    let offsets = mesh.polys.offsets();
    let conn = mesh.polys.connectivity();
    let pts = mesh.points.as_flat_slice();
    let (vx, vy, vz) = (view_direction[0], view_direction[1], view_direction[2]);

    let mut cell_depths = Vec::with_capacity(nc);
    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let n = (end - start) as f64;
        if n < 1.0 {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for idx in start..end {
            let b = conn[idx] as usize * 3;
            cx += pts[b];
            cy += pts[b + 1];
            cz += pts[b + 2];
        }
        let depth = (cx / n) * vx + (cy / n) * vy + (cz / n) * vz;
        cell_depths.push((ci, depth));
    }

    cell_depths.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    cell_depths
}

fn extract_cells_fast(mesh: &PolyData, cells: &[(usize, f64)]) -> PolyData {
    let offsets = mesh.polys.offsets();
    let conn = mesh.polys.connectivity();
    let np = mesh.points.len();
    let src_pts = mesh.points.as_flat_slice();

    let mut pt_map: Vec<i64> = vec![-1; np];
    let mut pts_flat: Vec<f64> = Vec::new();
    let mut new_conn = Vec::new();
    let mut new_off = Vec::with_capacity(cells.len() + 1);
    new_off.push(0i64);

    for &(ci, _) in cells {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        for idx in start..end {
            let old_id = conn[idx] as usize;
            if pt_map[old_id] < 0 {
                pt_map[old_id] = (pts_flat.len() / 3) as i64;
                let b = old_id * 3;
                pts_flat.extend_from_slice(&src_pts[b..b + 3]);
            }
            new_conn.push(pt_map[old_id]);
        }
        new_off.push(new_conn.len() as i64);
    }

    let mut result = PolyData::new();
    result.points = Points::from_flat_vec(pts_flat);
    result.polys = CellArray::from_raw(new_off, new_conn);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_two_planes() -> PolyData {
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.5, 1.0, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        )
    }

    #[test]
    fn depth_peel_two_layers() {
        let mesh = make_two_planes();
        let layers = depth_peel_layers(&mesh, [0.0, 0.0, 1.0], 2);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].polys.num_cells(), 1);
        assert_eq!(layers[1].polys.num_cells(), 1);
    }

    #[test]
    fn depth_sort() {
        let mesh = make_two_planes();
        let sorted = depth_sort_mesh(&mesh, [0.0, 0.0, 1.0]);
        assert_eq!(sorted.polys.num_cells(), 2);
        assert!(sorted.cell_data().get_array("Depth").is_some());
        let depth_arr = sorted.cell_data().get_array("Depth").unwrap();
        let mut d0 = [0.0f64];
        let mut d1 = [0.0f64];
        depth_arr.tuple_as_f64(0, &mut d0);
        depth_arr.tuple_as_f64(1, &mut d1);
        assert!(d0[0] <= d1[0]);
    }

    #[test]
    fn single_layer() {
        let mesh = make_two_planes();
        let layers = depth_peel_layers(&mesh, [0.0, 0.0, 1.0], 1);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].polys.num_cells(), 2);
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::new();
        let layers = depth_peel_layers(&mesh, [0.0, 0.0, 1.0], 4);
        assert!(layers.is_empty());
    }
}
