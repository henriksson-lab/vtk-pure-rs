use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-cell bounding boxes and add as cell data.
///
/// Adds a 6-component "CellBounds" array (x_min, x_max, y_min, y_max, z_min, z_max)
/// to cell data.
pub fn compute_cell_bounds(input: &PolyData) -> PolyData {
    let mut bounds_data = Vec::new();

    for cell in input.polys.iter() {
        if cell.is_empty() {
            bounds_data.extend_from_slice(&[0.0; 6]);
            continue;
        }

        let mut xmin = f64::MAX;
        let mut xmax = f64::MIN;
        let mut ymin = f64::MAX;
        let mut ymax = f64::MIN;
        let mut zmin = f64::MAX;
        let mut zmax = f64::MIN;

        for &id in cell {
            let p = input.points.get(id as usize);
            xmin = xmin.min(p[0]);
            xmax = xmax.max(p[0]);
            ymin = ymin.min(p[1]);
            ymax = ymax.max(p[1]);
            zmin = zmin.min(p[2]);
            zmax = zmax.max(p[2]);
        }

        bounds_data.extend_from_slice(&[xmin, xmax, ymin, ymax, zmin, zmax]);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CellBounds",
            bounds_data,
            6,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_bounds() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 1.0, 2.0], [3.0, 4.0, 5.0], [6.0, 7.0, 8.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_cell_bounds(&pd);
        let arr = result.cell_data().get_array("CellBounds").unwrap();
        assert_eq!(arr.num_components(), 6);
        let mut val = [0.0f64; 6];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 0.0).abs() < 1e-10); // x_min
        assert!((val[1] - 6.0).abs() < 1e-10); // x_max
        assert!((val[2] - 1.0).abs() < 1e-10); // y_min
        assert!((val[3] - 7.0).abs() < 1e-10); // y_max
    }

    #[test]
    fn two_cells() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 10.0],
                [11.0, 10.0, 10.0],
                [10.5, 11.0, 10.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = compute_cell_bounds(&pd);
        let arr = result.cell_data().get_array("CellBounds").unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }
}
