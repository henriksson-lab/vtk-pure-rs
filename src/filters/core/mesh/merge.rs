use crate::data::PolyData;

/// Merge two PolyData meshes into one by concatenating points and cells.
///
/// Cell point indices from the second mesh are adjusted by the number of points
/// in the first mesh. All cell types (verts, lines, polys, strips) are merged.
pub fn merge_poly_data(a: &PolyData, b: &PolyData) -> PolyData {
    crate::filters::core::mesh::mesh_copy::append_meshes(&[a, b])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn merge_two_triangles() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = merge_poly_data(&a, &b);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
        // Second triangle's indices should be offset by 3
        let cell1 = result.polys.cell(1);
        assert_eq!(cell1, &[3, 4, 5]);
    }

    #[test]
    fn merge_with_empty() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::new();
        let result = merge_poly_data(&a, &b);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn merge_preserves_all_points() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([5.0, 5.0, 5.0]);

        let result = merge_poly_data(&a, &b);
        assert_eq!(result.points.len(), 3);
        let p = result.points.get(2);
        assert!((p[0] - 5.0).abs() < 1e-10);
        assert!((p[1] - 5.0).abs() < 1e-10);
        assert!((p[2] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn merge_preserves_common_point_arrays() {
        let mut a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let mut b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![4.0, 5.0, 6.0],
                1,
            )));

        let result = merge_poly_data(&a, &b);
        let arr = result.point_data().get_array("temperature").unwrap();
        assert_eq!(arr.to_f64_vec(), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn merge_preserves_cell_data_in_output_cell_order() {
        let mut a = mixed_cell_mesh();
        let mut b = mixed_cell_mesh();
        add_cell_ids(&mut a, 0.0);
        add_cell_ids(&mut b, 10.0);

        let result = merge_poly_data(&a, &b);
        let arr = result.cell_data().get_array("cid").unwrap();
        assert_eq!(
            arr.to_f64_vec(),
            vec![0.0, 1.0, 2.0, 3.0, 10.0, 11.0, 12.0, 13.0]
        );
    }

    fn mixed_cell_mesh() -> PolyData {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);
        mesh
    }

    fn add_cell_ids(mesh: &mut PolyData, base: f64) {
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cid",
                (0..mesh.total_cells()).map(|i| base + i as f64).collect(),
                1,
            )));
    }
}
