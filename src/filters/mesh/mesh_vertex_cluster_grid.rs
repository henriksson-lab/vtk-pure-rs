//! Cluster vertices using a uniform grid (voxel-based simplification).
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::{HashMap, HashSet};

pub fn grid_cluster(mesh: &PolyData, cell_size: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || cell_size <= 0.0 || !cell_size.is_finite() {
        return mesh.clone();
    }
    let cs = cell_size;
    let mut grid: HashMap<(i64, i64, i64), usize> = HashMap::new();
    let mut clusters: Vec<([f64; 3], usize)> = Vec::new();
    let mut cluster_points: Vec<Vec<usize>> = Vec::new();
    let mut remap = vec![0usize; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        let key = (
            (p[0] / cs).floor() as i64,
            (p[1] / cs).floor() as i64,
            (p[2] / cs).floor() as i64,
        );
        let idx = *grid.entry(key).or_insert_with(|| {
            let idx = clusters.len();
            clusters.push(([0.0, 0.0, 0.0], 0));
            cluster_points.push(Vec::new());
            idx
        });
        clusters[idx].0[0] += p[0];
        clusters[idx].0[1] += p[1];
        clusters[idx].0[2] += p[2];
        clusters[idx].1 += 1;
        cluster_points[idx].push(i);
        remap[i] = idx;
    }
    let mut pts = Points::<f64>::new();
    for (sum, count) in clusters {
        let c = count as f64;
        pts.push([sum[0] / c, sum[1] / c, sum[2] / c]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    let mut old_cell_ids = Vec::new();
    let mut old_offset = 0usize;
    r.verts = remap_cell_array(&mesh.verts, &remap, 1, old_offset, &mut old_cell_ids);
    old_offset += mesh.verts.num_cells();
    r.lines = remap_cell_array(&mesh.lines, &remap, 2, old_offset, &mut old_cell_ids);
    old_offset += mesh.lines.num_cells();
    r.polys = remap_cell_array(&mesh.polys, &remap, 3, old_offset, &mut old_cell_ids);
    old_offset += mesh.polys.num_cells();
    r.strips = remap_cell_array(&mesh.strips, &remap, 3, old_offset, &mut old_cell_ids);
    remap_point_data(mesh, &cluster_points, &mut r);
    remap_cell_data(mesh, &old_cell_ids, &mut r);
    *r.field_data_mut() = mesh.field_data().clone();
    r
}

fn remap_cell_array(
    cells: &CellArray,
    remap: &[usize],
    min_unique: usize,
    old_offset: usize,
    old_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut out = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let mut mapped = Vec::with_capacity(cell.len());
        let mut valid = true;
        for &point_id in cell {
            if point_id < 0 || point_id as usize >= remap.len() {
                valid = false;
                break;
            }
            mapped.push(remap[point_id as usize] as i64);
        }
        if valid && mapped.iter().copied().collect::<HashSet<_>>().len() >= min_unique {
            out.push_cell(&mapped);
            old_cell_ids.push(old_offset + cell_id);
        }
    }
    out
}

fn remap_point_data(input: &PolyData, clusters: &[Vec<usize>], output: &mut PolyData) {
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(average_cluster_array(array, clusters));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn average_cluster_array(array: &AnyDataArray, clusters: &[Vec<usize>]) -> AnyDataArray {
    macro_rules! average {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(average_typed_cluster_array($array, clusters))
        };
    }

    match array {
        AnyDataArray::F32(array) => average!(array, F32),
        AnyDataArray::F64(array) => average!(array, F64),
        AnyDataArray::I8(array) => average!(array, I8),
        AnyDataArray::I16(array) => average!(array, I16),
        AnyDataArray::I32(array) => average!(array, I32),
        AnyDataArray::I64(array) => average!(array, I64),
        AnyDataArray::U8(array) => average!(array, U8),
        AnyDataArray::U16(array) => average!(array, U16),
        AnyDataArray::U32(array) => average!(array, U32),
        AnyDataArray::U64(array) => average!(array, U64),
    }
}

fn average_typed_cluster_array<T: Scalar>(
    array: &DataArray<T>,
    clusters: &[Vec<usize>],
) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(clusters.len() * num_components);
    for points in clusters {
        for component in 0..num_components {
            let sum: f64 = points
                .iter()
                .map(|&point_id| array.tuple(point_id)[component].to_f64())
                .sum();
            data.push(T::from_f64(sum / points.len() as f64));
        }
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn remap_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_cell_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_cell_tuples(array: &AnyDataArray, old_cell_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(select_typed_cell_tuples($array, old_cell_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => select!(array, F32),
        AnyDataArray::F64(array) => select!(array, F64),
        AnyDataArray::I8(array) => select!(array, I8),
        AnyDataArray::I16(array) => select!(array, I16),
        AnyDataArray::I32(array) => select!(array, I32),
        AnyDataArray::I64(array) => select!(array, I64),
        AnyDataArray::U8(array) => select!(array, U8),
        AnyDataArray::U16(array) => select!(array, U16),
        AnyDataArray::U32(array) => select!(array, U32),
        AnyDataArray::U64(array) => select!(array, U64),
    }
}

fn select_typed_cell_tuples<T: Scalar>(
    array: &DataArray<T>,
    old_cell_ids: &[usize],
) -> DataArray<T> {
    let mut data = Vec::with_capacity(old_cell_ids.len() * array.num_components());
    for &old_cell_id in old_cell_ids {
        data.extend_from_slice(array.tuple(old_cell_id));
    }
    DataArray::from_vec(array.name(), data, array.num_components())
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = grid_cluster(&m, 3.0);
        assert!(r.points.len() <= 3);
        assert!(r.polys.num_cells() <= 1);
    }

    #[test]
    fn skips_invalid_and_degenerate_cells() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([0.1, 0.0, 0.0]);
        m.points.push([0.0, 0.1, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.polys.push_cell(&[0, 1, 2]);
        m.polys.push_cell(&[0, -1, 3]);

        let r = grid_cluster(&m, 0.5);
        assert_eq!(r.polys.num_cells(), 0);
    }
}
