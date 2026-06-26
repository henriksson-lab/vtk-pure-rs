//! Spatial vertex clustering for mesh simplification and point cloud reduction.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::{BTreeMap, HashSet};

/// Cluster vertices into a regular grid and merge each cluster into one vertex.
///
/// This is a fast O(n) simplification method.
pub fn cluster_vertices_grid(mesh: &PolyData, grid_size: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || grid_size <= 0.0 || !grid_size.is_finite() {
        return mesh.clone();
    }

    // Assign each vertex to a grid cell
    let mut cell_map: BTreeMap<[i64; 3], Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let key = [
            (p[0] / grid_size).floor() as i64,
            (p[1] / grid_size).floor() as i64,
            (p[2] / grid_size).floor() as i64,
        ];
        cell_map.entry(key).or_default().push(i);
    }

    // Compute cluster centroids
    let mut new_points = Points::<f64>::new();
    let mut vertex_remap = vec![0usize; n];
    for (_, indices) in &cell_map {
        let new_idx = new_points.len();
        let mut avg = [0.0; 3];
        for &i in indices {
            let p = mesh.points.get(i);
            for c in 0..3 {
                avg[c] += p[c];
            }
        }
        let k = indices.len() as f64;
        new_points.push([avg[0] / k, avg[1] / k, avg[2] / k]);
        for &i in indices {
            vertex_remap[i] = new_idx;
        }
    }

    let mut result = PolyData::new();
    result.points = new_points;
    let mut old_cell_ids = Vec::new();
    let mut old_offset = 0usize;
    result.verts = remap_cell_array(&mesh.verts, &vertex_remap, 1, old_offset, &mut old_cell_ids);
    old_offset += mesh.verts.num_cells();
    result.lines = remap_cell_array(&mesh.lines, &vertex_remap, 2, old_offset, &mut old_cell_ids);
    old_offset += mesh.lines.num_cells();
    result.polys = remap_cell_array(&mesh.polys, &vertex_remap, 3, old_offset, &mut old_cell_ids);
    old_offset += mesh.polys.num_cells();
    result.strips = remap_cell_array(
        &mesh.strips,
        &vertex_remap,
        3,
        old_offset,
        &mut old_cell_ids,
    );
    remap_point_data(mesh, &cell_map, &mut result);
    remap_cell_data(mesh, &old_cell_ids, &mut result);
    for array in mesh.field_data().iter() {
        result.field_data_mut().add_array(array.clone());
    }
    result
}

fn remap_cell_array(
    cells: &CellArray,
    vertex_remap: &[usize],
    min_unique: usize,
    old_offset: usize,
    old_cell_ids: &mut Vec<usize>,
) -> CellArray {
    let mut remapped_cells = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        let mut valid = true;
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&pid| {
                if pid < 0 || pid as usize >= vertex_remap.len() {
                    valid = false;
                    0
                } else {
                    vertex_remap[pid as usize] as i64
                }
            })
            .collect();
        if valid && remapped.iter().copied().collect::<HashSet<_>>().len() >= min_unique {
            remapped_cells.push_cell(&remapped);
            old_cell_ids.push(old_offset + cell_id);
        }
    }
    remapped_cells
}

fn remap_point_data(
    input: &PolyData,
    cell_map: &BTreeMap<[i64; 3], Vec<usize>>,
    output: &mut PolyData,
) {
    output.point_data_mut().clear();
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(average_cluster_array(array, cell_map));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn average_cluster_array(
    array: &AnyDataArray,
    cell_map: &BTreeMap<[i64; 3], Vec<usize>>,
) -> AnyDataArray {
    macro_rules! average {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(average_typed_cluster_array($array, cell_map))
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
    cell_map: &BTreeMap<[i64; 3], Vec<usize>>,
) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(cell_map.len() * num_components);
    for indices in cell_map.values() {
        for component in 0..num_components {
            let sum: f64 = indices
                .iter()
                .map(|&idx| array.tuple(idx)[component].to_f64())
                .sum();
            data.push(T::from_f64(sum / indices.len() as f64));
        }
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn remap_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    output.cell_data_mut().clear();
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

/// Cluster a point cloud into N clusters using farthest point sampling.
pub fn farthest_point_cluster(mesh: &PolyData, n_clusters: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || n_clusters == 0 {
        return mesh.clone();
    }
    let nc = n_clusters.min(n);

    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut centers = vec![0usize; nc];
    let mut min_dist = vec![f64::MAX; n];
    centers[0] = 0;

    for ci in 1..nc {
        // Update min distances
        let prev = centers[ci - 1];
        for i in 0..n {
            let d = dist2(pts[i], pts[prev]);
            min_dist[i] = min_dist[i].min(d);
        }
        // Pick farthest point
        let farthest = (0..n)
            .max_by(|&a, &b| {
                min_dist[a]
                    .partial_cmp(&min_dist[b])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        centers[ci] = farthest;
    }

    // Assign each point to nearest center
    let mut labels = vec![0usize; n];
    for i in 0..n {
        let mut best = 0;
        let mut best_d = f64::MAX;
        for (ci, &c) in centers.iter().enumerate() {
            let d = dist2(pts[i], pts[c]);
            if d < best_d {
                best_d = d;
                best = ci;
            }
        }
        labels[i] = best;
    }

    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("ClusterId", data, 1)));
    result
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grid_cluster() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64 * 0.1, y as f64 * 0.1, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let simplified = cluster_vertices_grid(&mesh, 0.3);
        assert!(simplified.points.len() < mesh.points.len());
        assert!(simplified.points.len() > 0);
    }
    #[test]
    fn farthest_point() {
        let mesh = PolyData::from_points((0..50).map(|i| [i as f64, 0.0, 0.0]).collect::<Vec<_>>());
        let result = farthest_point_cluster(&mesh, 5);
        assert!(result.point_data().get_array("ClusterId").is_some());
    }

    #[test]
    fn grid_cluster_remaps_lines_and_data() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([0.1, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.1, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 2]);
        mesh.lines.push_cell(&[1, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "value",
                vec![2.0, 4.0, 10.0, 14.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("value");
        mesh.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![7, 9],
                1,
            )));

        let result = cluster_vertices_grid(&mesh, 0.5);

        assert_eq!(result.points.len(), 2);
        assert_eq!(result.lines.num_cells(), 2);
        let value = result.point_data().get_array("value").unwrap();
        assert_eq!(value.num_tuples(), 2);
        let mut scalar = [0.0];
        value.tuple_as_f64(0, &mut scalar);
        assert_eq!(scalar[0], 3.0);
        value.tuple_as_f64(1, &mut scalar);
        assert_eq!(scalar[0], 12.0);
        assert!(result.point_data().scalars().is_some());
        let cell_id = result.cell_data().get_array("cell_id").unwrap();
        assert_eq!(cell_id.num_tuples(), 2);
        cell_id.tuple_as_f64(1, &mut scalar);
        assert_eq!(scalar[0], 9.0);
    }
}
