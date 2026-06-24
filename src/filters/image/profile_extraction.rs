//! Extract 1D profiles from ImageData along lines and paths.

use crate::data::ImageData;
use crate::data::{AnyDataArray, DataArray, Table};

/// Extract a 1D profile along a row (fixed Y, Z).
pub fn extract_row_profile(image: &ImageData, array_name: &str, y: usize, z: usize) -> Table {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return Table::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let mut buf = [0.0f64];
    let mut x_data = Vec::new();
    let mut v_data = Vec::new();
    for ix in 0..dims[0] {
        let idx = ix + y * dims[0] + z * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            x_data.push(org[0] + ix as f64 * sp[0]);
            v_data.push(buf[0]);
        }
    }
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "Position", x_data, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            array_name, v_data, 1,
        )))
}

/// Extract a 1D profile along a column (fixed X, Z).
pub fn extract_column_profile(image: &ImageData, array_name: &str, x: usize, z: usize) -> Table {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return Table::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let mut buf = [0.0f64];
    let mut y_data = Vec::new();
    let mut v_data = Vec::new();
    for iy in 0..dims[1] {
        let idx = x + iy * dims[0] + z * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            y_data.push(org[1] + iy as f64 * sp[1]);
            v_data.push(buf[0]);
        }
    }
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "Position", y_data, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            array_name, v_data, 1,
        )))
}

/// Extract a 1D profile along a depth column (fixed X, Y).
pub fn extract_depth_profile(image: &ImageData, array_name: &str, x: usize, y: usize) -> Table {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return Table::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let org = image.origin();
    let mut buf = [0.0f64];
    let mut z_data = Vec::new();
    let mut v_data = Vec::new();
    for iz in 0..dims[2] {
        let idx = x + y * dims[0] + iz * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            z_data.push(org[2] + iz as f64 * sp[2]);
            v_data.push(buf[0]);
        }
    }
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "Position", z_data, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            array_name, v_data, 1,
        )))
}

/// Extract a diagonal profile from (0,0) to (nx-1,ny-1).
pub fn extract_diagonal_profile(image: &ImageData, array_name: &str, z: usize) -> Table {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return Table::new(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let n = dims[0].min(dims[1]);
    let mut buf = [0.0f64];
    let mut pos = Vec::new();
    let mut vals = Vec::new();
    for i in 0..n {
        let idx = i + i * dims[0] + z * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            pos.push(((i as f64 * sp[0]).powi(2) + (i as f64 * sp[1]).powi(2)).sqrt());
            vals.push(buf[0]);
        }
    }
    // Fix: compute position correctly
    let mut pos_correct = Vec::new();
    for i in 0..vals.len() {
        pos_correct.push(i as f64 * (sp[0] * sp[0] + sp[1] * sp[1]).sqrt());
    }
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "Position",
            pos_correct,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let table = extract_row_profile(&img, "v", 5, 0);
        assert_eq!(table.num_rows(), 10);
    }
    #[test]
    fn column() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, y, _| y,
        );
        let table = extract_column_profile(&img, "v", 5, 0);
        assert_eq!(table.num_rows(), 10);
    }
    #[test]
    fn depth() {
        let img = ImageData::from_function(
            [5, 5, 5],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, z| z,
        );
        let table = extract_depth_profile(&img, "v", 2, 2);
        assert_eq!(table.num_rows(), 5);
    }
    #[test]
    fn diagonal() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let table = extract_diagonal_profile(&img, "v", 0);
        assert_eq!(table.num_rows(), 8);
    }
}
