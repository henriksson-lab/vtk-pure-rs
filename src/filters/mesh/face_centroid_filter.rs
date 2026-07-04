use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use std::collections::HashMap;

/// Select faces whose centroid is inside a bounding box.
pub fn select_faces_in_box(input: &PolyData, bounds: [f64; 6]) -> PolyData {
    let mut pt_map: HashMap<usize, i64> = HashMap::new();
    let mut out_point_ids = Vec::new();
    let mut out_cell_ids = Vec::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for (ci, cell) in input.polys.iter().enumerate() {
        let Some(centroid) = cell_centroid(input, cell) else {
            continue;
        };

        if centroid[0] >= bounds[0]
            && centroid[0] <= bounds[1]
            && centroid[1] >= bounds[2]
            && centroid[1] <= bounds[3]
            && centroid[2] >= bounds[4]
            && centroid[2] <= bounds[5]
        {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    let id = usize::try_from(id).expect("validated by cell_centroid");
                    *pt_map.entry(id).or_insert_with(|| {
                        let i = out_pts.len() as i64;
                        out_pts.push(input.points.get(id));
                        out_point_ids.push(id);
                        i
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
            out_cell_ids.push(ci);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    copy_attribute_tuples(input.point_data(), pd.point_data_mut(), &out_point_ids);
    copy_poly_cell_attributes(input, pd.cell_data_mut(), &out_cell_ids);
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

/// Select faces whose centroid is inside a sphere.
pub fn select_faces_in_sphere(input: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let r2 = radius * radius;
    let mut pt_map: HashMap<usize, i64> = HashMap::new();
    let mut out_point_ids = Vec::new();
    let mut out_cell_ids = Vec::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for (ci, cell) in input.polys.iter().enumerate() {
        let Some(centroid) = cell_centroid(input, cell) else {
            continue;
        };

        let d2 = (centroid[0] - center[0]).powi(2)
            + (centroid[1] - center[1]).powi(2)
            + (centroid[2] - center[2]).powi(2);
        if d2 <= r2 {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    let id = usize::try_from(id).expect("validated by cell_centroid");
                    *pt_map.entry(id).or_insert_with(|| {
                        let i = out_pts.len() as i64;
                        out_pts.push(input.points.get(id));
                        out_point_ids.push(id);
                        i
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
            out_cell_ids.push(ci);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    copy_attribute_tuples(input.point_data(), pd.point_data_mut(), &out_point_ids);
    copy_poly_cell_attributes(input, pd.cell_data_mut(), &out_cell_ids);
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

fn cell_centroid(input: &PolyData, cell: &[i64]) -> Option<[f64; 3]> {
    if cell.is_empty() {
        return None;
    }

    let mut centroid = [0.0; 3];
    for &id in cell {
        let idx = valid_point_index(id, input.points.len())?;
        let p = input.points.get(idx);
        centroid[0] += p[0];
        centroid[1] += p[1];
        centroid[2] += p[2];
    }

    let n = cell.len() as f64;
    Some([centroid[0] / n, centroid[1] / n, centroid[2] / n])
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn copy_poly_cell_attributes(input: &PolyData, target: &mut DataSetAttributes, poly_ids: &[usize]) {
    let poly_offset = input.verts.num_cells() + input.lines.num_cells();
    for array in input.cell_data().iter() {
        let ids: Vec<usize> = if array.num_tuples() >= input.total_cells() {
            poly_ids.iter().map(|&id| id + poly_offset).collect()
        } else {
            poly_ids.to_vec()
        };
        if ids.iter().all(|&id| id < array.num_tuples()) {
            target.add_array(subset_array(array, &ids));
        }
    }
    copy_active_attributes(input.cell_data(), target);
}

fn copy_attribute_tuples(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if tuple_ids.iter().all(|&id| id < array.num_tuples()) {
            target.add_array(subset_array(array, tuple_ids));
        }
    }
    copy_active_attributes(source, target);
}

fn copy_active_attributes(source: &DataSetAttributes, target: &mut DataSetAttributes) {
    if let Some(array) = source.scalars() {
        target.set_active_scalars(array.name());
    }
    if let Some(array) = source.vectors() {
        target.set_active_vectors(array.name());
    }
    if let Some(array) = source.normals() {
        target.set_active_normals(array.name());
    }
    if let Some(array) = source.tcoords() {
        target.set_active_tcoords(array.name());
    }
    if let Some(array) = source.tensors() {
        target.set_active_tensors(array.name());
    }
    if let Some(array) = source.global_ids() {
        target.set_active_global_ids(array.name());
    }
    if let Some(array) = source.pedigree_ids() {
        target.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = source.edge_flags() {
        target.set_active_edge_flags(array.name());
    }
    if let Some(array) = source.tangents() {
        target.set_active_tangents(array.name());
    }
    if let Some(array) = source.rational_weights() {
        target.set_active_rational_weights(array.name());
    }
    if let Some(array) = source.higher_order_degrees() {
        target.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = source.process_ids() {
        target.set_active_process_ids(array.name());
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! subset {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut values = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                values.extend_from_slice($arr.tuple(tuple_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($arr.name(), values, nc))
        }};
    }
    match array {
        AnyDataArray::F32(arr) => subset!(arr, F32),
        AnyDataArray::F64(arr) => subset!(arr, F64),
        AnyDataArray::I8(arr) => subset!(arr, I8),
        AnyDataArray::I16(arr) => subset!(arr, I16),
        AnyDataArray::I32(arr) => subset!(arr, I32),
        AnyDataArray::I64(arr) => subset!(arr, I64),
        AnyDataArray::U8(arr) => subset!(arr, U8),
        AnyDataArray::U16(arr) => subset!(arr, U16),
        AnyDataArray::U32(arr) => subset!(arr, U32),
        AnyDataArray::U64(arr) => subset!(arr, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_in_box() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]); // centroid ~(0.5,0.33,0)
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]); // centroid ~(10.5,0.33,0)
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = select_faces_in_box(&pd, [0.0, 2.0, 0.0, 2.0, -1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1); // only first face
    }

    #[test]
    fn select_in_sphere() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = select_faces_in_sphere(&pd, [0.5, 0.5, 0.0], 2.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn select_none() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = select_faces_in_box(&pd, [100.0, 200.0, 100.0, 200.0, 100.0, 200.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(select_faces_in_box(&pd, [0.0; 6]).polys.num_cells(), 0);
    }

    #[test]
    fn invalid_polygon_ids_are_skipped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = select_faces_in_box(&pd, [-1.0, 2.0, -1.0, 2.0, -1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
