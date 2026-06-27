use crate::data::{AnyDataArray, DataArray, ImageData};

/// Crop an ImageData to a sub-extent specified by index ranges.
///
/// Extracts the region [i0..i1, j0..j1, k0..k1] (inclusive) from the input.
pub fn image_crop(
    input: &ImageData,
    scalars: &str,
    i_range: [usize; 2],
    j_range: [usize; 2],
    k_range: [usize; 2],
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    if nx == 0 || ny == 0 || nz == 0 {
        return ImageData::new();
    }
    let origin = input.origin();
    let spacing = input.spacing();

    let i0 = i_range[0].min(nx - 1);
    let i1 = i_range[1].min(nx - 1);
    let j0 = j_range[0].min(ny - 1);
    let j1 = j_range[1].min(ny - 1);
    let k0 = k_range[0].min(nz - 1);
    let k1 = k_range[1].min(nz - 1);
    if i0 > i1 || j0 > j1 || k0 > k1 {
        return ImageData::new();
    }

    let new_nx = i1 - i0 + 1;
    let new_ny = j1 - j0 + 1;
    let new_nz = k1 - k0 + 1;
    let num_components = arr.num_components();

    let new_origin = [
        origin[0] + i0 as f64 * spacing[0],
        origin[1] + j0 as f64 * spacing[1],
        origin[2] + k0 as f64 * spacing[2],
    ];

    let mut values = Vec::with_capacity(new_nx * new_ny * new_nz * num_components);
    let mut buf = vec![0.0f64; num_components];

    for k in k0..=k1 {
        for j in j0..=j1 {
            for i in i0..=i1 {
                let src_idx = k * ny * nx + j * nx + i;
                arr.tuple_as_f64(src_idx, &mut buf);
                values.extend_from_slice(&buf);
            }
        }
    }

    let mut img = ImageData::with_dimensions(new_nx, new_ny, new_nz);
    img.set_origin(new_origin);
    img.set_spacing(spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            values,
            num_components,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_center() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let values: Vec<f64> = (0..125).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_crop(&img, "v", [1, 3], [1, 3], [1, 3]);
        assert_eq!(result.dimensions(), [3, 3, 3]);
        assert!(result.point_data().get_array("v").is_some());
    }

    #[test]
    fn crop_single_slice() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let values: Vec<f64> = (0..125).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_crop(&img, "v", [0, 4], [0, 4], [2, 2]);
        assert_eq!(result.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn full_crop_unchanged() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let values: Vec<f64> = (0..27).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_crop(&img, "v", [0, 2], [0, 2], [0, 2]);
        assert_eq!(result.dimensions(), [3, 3, 3]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(26, &mut buf);
        assert_eq!(buf[0], 26.0);
    }
}
