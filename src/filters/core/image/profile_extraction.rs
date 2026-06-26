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
    if y >= dims[1] || z >= dims[2] {
        return Table::new();
    }
    let mut buf = [0.0f64];
    let mut x_data = Vec::new();
    let mut v_data = Vec::new();
    for ix in 0..dims[0] {
        let idx = ix + y * dims[0] + z * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            x_data.push(image.point_from_ijk(ix, y, z)[0]);
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
    if x >= dims[0] || z >= dims[2] {
        return Table::new();
    }
    let mut buf = [0.0f64];
    let mut y_data = Vec::new();
    let mut v_data = Vec::new();
    for iy in 0..dims[1] {
        let idx = x + iy * dims[0] + z * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            y_data.push(image.point_from_ijk(x, iy, z)[1]);
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
    if x >= dims[0] || y >= dims[1] {
        return Table::new();
    }
    let mut buf = [0.0f64];
    let mut z_data = Vec::new();
    let mut v_data = Vec::new();
    for iz in 0..dims[2] {
        let idx = x + y * dims[0] + iz * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            z_data.push(image.point_from_ijk(x, y, iz)[2]);
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
    if z >= dims[2] {
        return Table::new();
    }
    let n = dims[0].min(dims[1]);
    let mut buf = [0.0f64];
    let mut vals = Vec::new();
    let first = image.point_from_ijk(0, 0, z);
    for i in 0..n {
        let idx = i + i * dims[0] + z * dims[0] * dims[1];
        if idx < arr.num_tuples() {
            arr.tuple_as_f64(idx, &mut buf);
            vals.push(buf[0]);
        }
    }
    let mut pos = Vec::with_capacity(vals.len());
    for i in 0..vals.len() {
        let p = image.point_from_ijk(i, i, z);
        let dx = p[0] - first[0];
        let dy = p[1] - first[1];
        let dz = p[2] - first[2];
        pos.push((dx * dx + dy * dy + dz * dz).sqrt());
    }
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec("Position", pos, 1)))
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

    #[test]
    fn out_of_bounds_fixed_coordinate_returns_empty_table() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        assert_eq!(extract_row_profile(&img, "v", 4, 0).num_rows(), 0);
        assert_eq!(extract_column_profile(&img, "v", 0, 1).num_rows(), 0);
        assert_eq!(extract_diagonal_profile(&img, "v", 1).num_rows(), 0);
    }

    #[test]
    fn row_profile_positions_include_extent_offset() {
        let mut img = ImageData::with_dimensions(3, 2, 1);
        img.set_extent([10, 12, 20, 21, 0, 0]);
        img.set_spacing([2.0, 3.0, 1.0]);
        img.set_origin([5.0, 7.0, 0.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0; 6],
                1,
            )));

        let table = extract_row_profile(&img, "v", 0, 0);
        assert_eq!(table.value_f64(0, "Position"), Some(25.0));
        assert_eq!(table.value_f64(2, "Position"), Some(29.0));
    }
}
