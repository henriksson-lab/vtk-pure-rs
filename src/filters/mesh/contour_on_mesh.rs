use crate::data::PolyData;

/// Extract multiple isocontours from a scalar field on a mesh.
///
/// Like `mesh_level_set` but for multiple values at once.
/// Returns a PolyData with line segments for all contours.
///
/// Re-exported from [`crate::filters::mesh::contour_lines`], which holds the
/// single implementation (it additionally tags each output line with its
/// isovalue in cell data).
pub use crate::filters::mesh::contour_lines::multi_contour_on_mesh;

/// Extract a single contour and compute its total length.
pub fn contour_length(input: &PolyData, array_name: &str, isovalue: f64) -> f64 {
    let contour = multi_contour_on_mesh(input, array_name, &[isovalue]);
    let mut total = 0.0;
    for cell in contour.lines.iter() {
        if cell.len() >= 2 {
            let a = contour.points.get(cell[0] as usize);
            let b = contour.points.get(cell[1] as usize);
            total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn contour_length_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 3.0, 0.0],
                1,
            )));

        let len = contour_length(&pd, "f", 1.0);
        assert!(len > 0.0, "contour length={}", len);
    }

    #[test]
    fn contour_through_exact_vertex() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.5],
                1,
            )));

        let result = multi_contour_on_mesh(&pd, "f", &[0.5]);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn no_crossing() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vec![5.0; 3], 1)));

        assert_eq!(contour_length(&pd, "f", 0.0), 0.0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert_eq!(
            multi_contour_on_mesh(&pd, "nope", &[1.0]).lines.num_cells(),
            0
        );
    }

    #[test]
    fn invalid_cell_is_skipped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0],
                1,
            )));

        assert_eq!(multi_contour_on_mesh(&pd, "f", &[0.5]).lines.num_cells(), 0);
    }
}
