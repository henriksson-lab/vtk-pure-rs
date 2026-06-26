//! Process/rank ID labeling for parallel visualization.
//!
//! Adds scalar arrays to identify which process or partition owns each
//! point or cell, useful for visualizing domain decomposition.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Add a process-id point data array with a constant value for all points.
pub fn add_process_id(mesh: &PolyData, process_id: usize) -> PolyData {
    let n = mesh.points.len();
    let data = vec![process_id as i64; n];
    let mut result = mesh.clone();
    let attrs = result.point_data_mut();
    attrs.add_array(AnyDataArray::I64(DataArray::from_vec(
        "PointProcessIds",
        data,
        1,
    )));
    attrs.set_active_process_ids("PointProcessIds");
    result
}

/// Add a process-id cell data array with a constant value for all cells.
pub fn add_process_id_cells(mesh: &PolyData, process_id: usize) -> PolyData {
    let n = mesh.total_cells();
    let data = vec![process_id as i64; n];
    let mut result = mesh.clone();
    let attrs = result.cell_data_mut();
    attrs.add_array(AnyDataArray::I64(DataArray::from_vec(
        "CellProcessIds",
        data,
        1,
    )));
    attrs.set_active_process_ids("CellProcessIds");
    result
}

/// Tag multiple meshes with sequential process IDs and merge.
pub fn tag_and_merge(meshes: &[&PolyData]) -> PolyData {
    if meshes.is_empty() {
        return PolyData::new();
    }
    let tagged: Vec<PolyData> = meshes
        .iter()
        .enumerate()
        .map(|(i, m)| add_process_id_cells(&add_process_id(m, i), i))
        .collect();
    let refs: Vec<&PolyData> = tagged.iter().collect();
    let mut output = crate::filters::core::append::append(&refs);
    output
        .point_data_mut()
        .set_active_process_ids("PointProcessIds");
    output
        .cell_data_mut()
        .set_active_process_ids("CellProcessIds");
    output
}

/// Split a mesh into N partitions and tag with ProcessId.
pub fn partition_and_tag(mesh: &PolyData, n_partitions: usize) -> PolyData {
    if mesh.points.len() == 0 || n_partitions == 0 {
        return mesh.clone();
    }

    // Simple round-robin point assignment
    let n = mesh.points.len();
    let data: Vec<i64> = (0..n).map(|i| (i % n_partitions) as i64).collect();
    let mut result = mesh.clone();
    let attrs = result.point_data_mut();
    attrs.add_array(AnyDataArray::I64(DataArray::from_vec(
        "PointProcessIds",
        data,
        1,
    )));
    attrs.set_active_process_ids("PointProcessIds");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_id() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let result = add_process_id(&mesh, 3);
        let arr = result.point_data().process_ids().unwrap();
        assert_eq!(arr.name(), "PointProcessIds");
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 3.0);
    }

    #[test]
    fn tag_merge() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0]]);
        let result = tag_and_merge(&[&a, &b]);
        assert_eq!(result.points.len(), 2);
        assert!(result.point_data().process_ids().is_some());
        assert!(result.cell_data().process_ids().is_some());
    }

    #[test]
    fn partition() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]);
        let result = partition_and_tag(&mesh, 2);
        let arr = result.point_data().process_ids().unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
