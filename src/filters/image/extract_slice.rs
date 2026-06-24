use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a 2D slice from a 3D ImageData along the Z axis at index `k`.
///
/// Returns a new ImageData with dimensions (nx, ny, 1).
pub fn extract_slice_z(input: &ImageData, scalars: &str, k: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return ImageData::with_dimensions(1, 1, 1),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let k = k.min(nz.saturating_sub(1));

    let ncomp: usize = arr.num_components();
    let mut values: Vec<f64> = Vec::with_capacity(nx * ny * ncomp);
    let mut buf: Vec<f64> = vec![0.0; ncomp];

    for j in 0..ny {
        for i in 0..nx {
            let src_idx: usize = k * ny * nx + j * nx + i;
            arr.tuple_as_f64(src_idx, &mut buf);
            values.extend_from_slice(&buf);
        }
    }

    let spacing = input.spacing();
    let origin = input.origin();
    let new_origin: [f64; 3] = [origin[0], origin[1], origin[2] + k as f64 * spacing[2]];

    let mut img = ImageData::with_dimensions(nx, ny, 1);
    img.set_spacing(spacing);
    img.set_origin(new_origin);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, ncomp,
        )));
    img
}

/// Extract a 2D slice from a 3D ImageData along the Y axis at index `j`.
///
/// Returns a new ImageData with dimensions (nx, 1, nz).
pub fn extract_slice_y(input: &ImageData, scalars: &str, j: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return ImageData::with_dimensions(1, 1, 1),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let j = j.min(ny.saturating_sub(1));

    let ncomp: usize = arr.num_components();
    let mut values: Vec<f64> = Vec::with_capacity(nx * nz * ncomp);
    let mut buf: Vec<f64> = vec![0.0; ncomp];

    for k in 0..nz {
        for i in 0..nx {
            let src_idx: usize = k * ny * nx + j * nx + i;
            arr.tuple_as_f64(src_idx, &mut buf);
            values.extend_from_slice(&buf);
        }
    }

    let spacing = input.spacing();
    let origin = input.origin();
    let new_origin: [f64; 3] = [origin[0], origin[1] + j as f64 * spacing[1], origin[2]];

    let mut img = ImageData::with_dimensions(nx, 1, nz);
    img.set_spacing(spacing);
    img.set_origin(new_origin);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, ncomp,
        )));
    img
}

/// Extract a 2D slice from a 3D ImageData along the X axis at index `i`.
///
/// Returns a new ImageData with dimensions (1, ny, nz).
pub fn extract_slice_x(input: &ImageData, scalars: &str, i: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return ImageData::with_dimensions(1, 1, 1),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let i = i.min(nx.saturating_sub(1));

    let ncomp: usize = arr.num_components();
    let mut values: Vec<f64> = Vec::with_capacity(ny * nz * ncomp);
    let mut buf: Vec<f64> = vec![0.0; ncomp];

    for k in 0..nz {
        for j in 0..ny {
            let src_idx: usize = k * ny * nx + j * nx + i;
            arr.tuple_as_f64(src_idx, &mut buf);
            values.extend_from_slice(&buf);
        }
    }

    let spacing = input.spacing();
    let origin = input.origin();
    let new_origin: [f64; 3] = [origin[0] + i as f64 * spacing[0], origin[1], origin[2]];

    let mut img = ImageData::with_dimensions(1, ny, nz);
    img.set_spacing(spacing);
    img.set_origin(new_origin);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, ncomp,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image() -> ImageData {
        let mut img = ImageData::with_dimensions(4, 3, 5);
        let n: usize = 4 * 3 * 5;
        let values: Vec<f64> = (0..n).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", values, 1)));
        img
    }

    #[test]
    fn extract_z_slice_dimensions() {
        let img = make_test_image();
        let result = extract_slice_z(&img, "data", 2);
        assert_eq!(result.dimensions(), [4, 3, 1]);
        assert!(result.point_data().get_array("data").is_some());
        let arr = result.point_data().get_array("data").unwrap();
        assert_eq!(arr.num_tuples(), 4 * 3);
    }

    #[test]
    fn extract_y_slice_dimensions() {
        let img = make_test_image();
        let result = extract_slice_y(&img, "data", 1);
        assert_eq!(result.dimensions(), [4, 1, 5]);
        let arr = result.point_data().get_array("data").unwrap();
        assert_eq!(arr.num_tuples(), 4 * 5);
    }

    #[test]
    fn extract_x_slice_dimensions() {
        let img = make_test_image();
        let result = extract_slice_x(&img, "data", 0);
        assert_eq!(result.dimensions(), [1, 3, 5]);
        let arr = result.point_data().get_array("data").unwrap();
        assert_eq!(arr.num_tuples(), 3 * 5);
    }
}
