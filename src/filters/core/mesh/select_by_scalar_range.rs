//! Select mesh cells by scalar value range.

use crate::data::{CellArray, Points, PolyData};

/// Extract cells whose point scalars are all within [lo, hi].
pub fn select_cells_by_scalar_range(
    mesh: &PolyData,
    array_name: &str,
    lo: f64,
    hi: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut used = vec![false; mesh.points.len()];
    let mut kept_cells: Vec<Vec<i64>> = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        if !cell.iter().all(|&v| (v as usize) < vals.len()) {
            continue;
        }
        if cell
            .iter()
            .all(|&v| vals[v as usize] >= lo && vals[v as usize] <= hi)
        {
            for &v in cell {
                used[v as usize] = true;
            }
            kept_cells.push(cell.to_vec());
        }
    }

    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept_cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        polys.push_cell(&mapped);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Extract points whose scalar is within [lo, hi] as vertices.
pub fn select_points_by_scalar_range(
    mesh: &PolyData,
    array_name: &str,
    lo: f64,
    hi: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= lo && buf[0] <= hi {
            let idx = pts.len();
            pts.push(mesh.points.get(i));
            verts.push_cell(&[idx as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_select_cells() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0, 8.0, 9.0],
                1,
            )));
        let result = select_cells_by_scalar_range(&mesh, "s", 0.0, 5.0);
        assert_eq!(result.polys.num_cells(), 1); // only first tri has all point scalars <= 5
    }
    #[test]
    fn test_select_points() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 5.0, 10.0],
                1,
            )));
        let result = select_points_by_scalar_range(&mesh, "s", 3.0, 7.0);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn missing_scalar_returns_empty() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        assert_eq!(
            select_cells_by_scalar_range(&mesh, "missing", 0.0, 1.0)
                .polys
                .num_cells(),
            0
        );
        assert_eq!(
            select_points_by_scalar_range(&mesh, "missing", 0.0, 1.0)
                .points
                .len(),
            0
        );
    }

    #[test]
    fn cells_with_missing_point_scalar_are_skipped() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.5, 0.5],
                1,
            )));

        let result = select_cells_by_scalar_range(&mesh, "s", 0.0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn cell_requires_all_point_scalars_in_range() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 0.5, 2.5],
                1,
            )));

        let result = select_cells_by_scalar_range(&mesh, "s", 0.0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
