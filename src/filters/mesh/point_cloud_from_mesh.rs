use crate::data::{CellArray, PolyData};

/// Controls how point-cloud vertex cells are generated.
///
/// Mirrors `vtkConvertToPointCloud::CellGeneration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointCloudCellGeneration {
    /// Do not generate any cells.
    NoCells,
    /// Generate one polyvertex cell containing every point.
    PolyVertexCell,
    /// Generate one vertex cell per point.
    VertexCells,
}

/// Convert a mesh to a point cloud using VTK's default polyvertex-cell mode.
pub fn point_cloud_from_mesh(input: &PolyData) -> PolyData {
    convert_to_point_cloud(input, PointCloudCellGeneration::PolyVertexCell)
}

/// Convert mesh points to a point-cloud `PolyData`.
///
/// This is the `PolyData` equivalent of VTK's `vtkConvertToPointCloud`: points,
/// point data, and field data are preserved; topology is replaced according to
/// the selected cell-generation mode.
pub fn convert_to_point_cloud(input: &PolyData, mode: PointCloudCellGeneration) -> PolyData {
    let mut output = PolyData::new();
    output.points = input.points.clone();
    *output.point_data_mut() = input.point_data().clone();
    *output.field_data_mut() = input.field_data().clone();

    match mode {
        PointCloudCellGeneration::NoCells => {}
        PointCloudCellGeneration::PolyVertexCell => {
            let ids: Vec<i64> = (0..input.points.len()).map(|i| i as i64).collect();
            output.verts.push_cell(&ids);
        }
        PointCloudCellGeneration::VertexCells => {
            let mut verts = CellArray::new();
            for i in 0..input.points.len() {
                verts.push_cell(&[i as i64]);
            }
            output.verts = verts;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    fn sample_mesh() -> PolyData {
        PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn default_generates_single_polyvertex_cell() {
        let mesh = sample_mesh();
        let result = point_cloud_from_mesh(&mesh);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.verts.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn no_cells_mode_preserves_points_without_topology() {
        let mesh = sample_mesh();
        let result = convert_to_point_cloud(&mesh, PointCloudCellGeneration::NoCells);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.verts.num_cells(), 0);
        assert_eq!(result.total_cells(), 0);
    }

    #[test]
    fn vertex_cells_mode_generates_one_cell_per_point() {
        let mesh = sample_mesh();
        let result = convert_to_point_cloud(&mesh, PointCloudCellGeneration::VertexCells);
        assert_eq!(result.verts.num_cells(), 3);
        assert_eq!(result.verts.cell(0), &[0]);
        assert_eq!(result.verts.cell(1), &[1]);
        assert_eq!(result.verts.cell(2), &[2]);
    }

    #[test]
    fn preserves_point_and_field_data() {
        let mut mesh = sample_mesh();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("temperature");
        mesh.field_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "time",
                vec![1.25],
                1,
            )));

        let result = point_cloud_from_mesh(&mesh);
        assert!(result.point_data().get_array("temperature").is_some());
        assert!(result.point_data().scalars().is_some());
        assert!(result.field_data().get_array("time").is_some());
    }
}
