//! Extract surface regions based on scalar value ranges.
//!
//! Combines threshold + extract_surface into a single operation.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract surface cells where a scalar is within [min_val, max_val].
///
/// More efficient than threshold + extract as it does both in one pass.
pub fn extract_surface_by_scalar(
    mesh: &PolyData,
    array_name: &str,
    min_val: f64,
    max_val: f64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };

    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut scalar_data: Vec<f64> = Vec::new();
    let mut buf = [0.0f64];

    for cell in mesh.polys.iter() {
        // Check if all vertices are within range
        let all_in = cell.iter().all(|&pid| {
            arr.tuple_as_f64(pid as usize, &mut buf);
            buf[0] >= min_val && buf[0] <= max_val
        });
        if !all_in {
            continue;
        }

        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *pt_map.entry(old).or_insert_with(|| {
                let idx = new_points.len();
                new_points.push(mesh.points.get(old));
                arr.tuple_as_f64(old, &mut buf);
                scalar_data.push(buf[0]);
                idx
            });
            new_ids.push(new_idx as i64);
        }
        new_polys.push_cell(&new_ids);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            scalar_data,
            1,
        )));
    result
}

/// Extract surface cells where ANY vertex scalar exceeds threshold.
pub fn extract_surface_above_scalar(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];

    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for cell in mesh.polys.iter() {
        let any_above = cell.iter().any(|&pid| {
            arr.tuple_as_f64(pid as usize, &mut buf);
            buf[0] >= threshold
        });
        if !any_above {
            continue;
        }

        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *pt_map.entry(old).or_insert_with(|| {
                let idx = new_points.len();
                new_points.push(mesh.points.get(old));
                idx
            });
            new_ids.push(new_idx as i64);
        }
        new_polys.push_cell(&new_ids);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh() -> PolyData {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 10.0, 10.0, 50.0, 50.0, 50.0],
                1,
            )));
        m
    }

    #[test]
    fn by_range() {
        let result = extract_surface_by_scalar(&make_mesh(), "temp", 0.0, 20.0);
        assert_eq!(result.polys.num_cells(), 1); // only first triangle
    }

    #[test]
    fn above_threshold() {
        let result = extract_surface_above_scalar(&make_mesh(), "temp", 40.0);
        assert_eq!(result.polys.num_cells(), 1); // only second triangle
    }

    #[test]
    fn all_pass() {
        let result = extract_surface_by_scalar(&make_mesh(), "temp", 0.0, 100.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn none_pass() {
        let result = extract_surface_by_scalar(&make_mesh(), "temp", 90.0, 100.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
