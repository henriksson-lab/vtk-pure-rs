//! Centroid-based operations: per-cell centroid, centroid-distance, centroid cloud.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-cell centroid as a point cloud.
///
/// Re-exported from [`crate::filters::mesh::vertex_to_polydata`], which holds the
/// single implementation (the faithful `vtkCellCenters` translation).
pub use crate::filters::mesh::vertex_to_polydata::cell_centroids_as_points;

/// Center mesh at origin (translate so centroid is at `[0, 0, 0]`).
///
/// Re-exported from [`crate::filters::mesh::transform_mesh`].
pub use crate::filters::mesh::transform_mesh::center_at_origin;

/// Normalize mesh to fit within a unit sphere centered at origin.
///
/// Re-exported from [`crate::filters::mesh::mesh_center_scale`].
pub use crate::filters::mesh::mesh_center_scale::normalize_to_unit_sphere;

/// Add distance from each vertex to the mesh centroid as point data.
pub fn distance_from_centroid(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CentroidDistance",
            data,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    #[test]
    fn centroids_include_vtk_polydata_cell_order() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        ]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[1, 3, 2]);
        let result = cell_centroids_as_points(&mesh);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
        assert_eq!(result.points.get(1), [1.0, 0.0, 0.0]);
        assert!((result.points.get(2)[0] - 2.0 / 3.0).abs() < 1e-10);
        assert!((result.points.get(3)[1] - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn centroids_copy_cell_data_to_point_data() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![10, 11],
                1,
            )));

        let result = cell_centroids_as_points(&mesh);
        let arr = result.point_data().get_array("cell_id").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 11.0);
    }

    #[test]
    fn dist() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = distance_from_centroid(&mesh);
        let arr = result.point_data().get_array("CentroidDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // distance from centroid (1,0,0) to (0,0,0)
    }
}
