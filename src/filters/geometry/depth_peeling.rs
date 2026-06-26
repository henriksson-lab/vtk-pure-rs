//! Depth-sorted polygon layers for order-independent transparency.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

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

    // Rebuild PolyData with sorted cell order using raw offsets/connectivity.
    // Like vtkDepthSortPolyData, point ids stay unchanged and data arrays are
    // copied in the new cell order.
    let offsets = mesh.polys.offsets();
    let conn = mesh.polys.connectivity();

    let total_conn = conn.len();
    let mut new_conn = Vec::with_capacity(total_conn);
    let mut new_off = Vec::with_capacity(nc + 1);
    new_off.push(0i64);
    let mut depth_arr = Vec::with_capacity(nc);
    let mut cell_ids = Vec::with_capacity(nc);

    for &(ci, depth) in &cell_depths {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        for idx in start..end {
            new_conn.push(conn[idx]);
        }
        new_off.push(new_conn.len() as i64);
        cell_ids.push(ci);
        depth_arr.push(depth);
    }

    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.polys = CellArray::from_raw(new_off, new_conn);
    *result.point_data_mut() = mesh.point_data().clone();
    copy_arrays_by_indices(mesh.cell_data(), result.cell_data_mut(), &cell_ids);
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
    let mut point_ids = Vec::new();
    let mut cell_ids = Vec::with_capacity(cells.len());
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
                point_ids.push(old_id);
                let b = old_id * 3;
                pts_flat.extend_from_slice(&src_pts[b..b + 3]);
            }
            new_conn.push(pt_map[old_id]);
        }
        new_off.push(new_conn.len() as i64);
        cell_ids.push(ci);
    }

    let mut result = PolyData::new();
    result.points = Points::from_flat_vec(pts_flat);
    result.polys = CellArray::from_raw(new_off, new_conn);
    copy_arrays_by_indices(mesh.point_data(), result.point_data_mut(), &point_ids);
    copy_arrays_by_indices(mesh.cell_data(), result.cell_data_mut(), &cell_ids);
    result
}

fn copy_arrays_by_indices(
    input: &DataSetAttributes,
    output: &mut DataSetAttributes,
    indices: &[usize],
) {
    for arr in input.iter() {
        output.add_array(copy_array_by_indices(arr, indices));
    }
    preserve_active_attributes(input, output);
}

fn copy_array_by_indices(arr: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {{
            AnyDataArray::$variant(copy_typed_array($array, indices))
        }};
    }
    match arr {
        AnyDataArray::F32(a) => copy!(a, F32),
        AnyDataArray::F64(a) => copy!(a, F64),
        AnyDataArray::I8(a) => copy!(a, I8),
        AnyDataArray::I16(a) => copy!(a, I16),
        AnyDataArray::I32(a) => copy!(a, I32),
        AnyDataArray::I64(a) => copy!(a, I64),
        AnyDataArray::U8(a) => copy!(a, U8),
        AnyDataArray::U16(a) => copy!(a, U16),
        AnyDataArray::U32(a) => copy!(a, U32),
        AnyDataArray::U64(a) => copy!(a, U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * nc);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, nc)
}

fn preserve_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(arr) = input.scalars() {
        output.set_active_scalars(arr.name());
    }
    if let Some(arr) = input.vectors() {
        output.set_active_vectors(arr.name());
    }
    if let Some(arr) = input.normals() {
        output.set_active_normals(arr.name());
    }
    if let Some(arr) = input.tcoords() {
        output.set_active_tcoords(arr.name());
    }
    if let Some(arr) = input.tensors() {
        output.set_active_tensors(arr.name());
    }
    if let Some(arr) = input.global_ids() {
        output.set_active_global_ids(arr.name());
    }
    if let Some(arr) = input.pedigree_ids() {
        output.set_active_pedigree_ids(arr.name());
    }
    if let Some(arr) = input.edge_flags() {
        output.set_active_edge_flags(arr.name());
    }
    if let Some(arr) = input.tangents() {
        output.set_active_tangents(arr.name());
    }
    if let Some(arr) = input.rational_weights() {
        output.set_active_rational_weights(arr.name());
    }
    if let Some(arr) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(arr.name());
    }
    if let Some(arr) = input.process_ids() {
        output.set_active_process_ids(arr.name());
    }
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
        let mut mesh = make_two_planes();
        mesh.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "PointIds",
                vec![0, 1, 2, 3, 4, 5],
                1,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "OriginalCellIds",
                vec![10, 20],
                1,
            )));

        let sorted = depth_sort_mesh(&mesh, [0.0, 0.0, -1.0]);
        assert_eq!(sorted.polys.num_cells(), 2);
        assert!(sorted.point_data().get_array("PointIds").is_some());
        let original_cell_ids = sorted.cell_data().get_array("OriginalCellIds").unwrap();
        let mut original_id = [0.0f64];
        original_cell_ids.tuple_as_f64(0, &mut original_id);
        assert_eq!(original_id[0], 20.0);
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
