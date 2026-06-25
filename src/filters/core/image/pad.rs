use crate::data::{AnyDataArray, DataArray, ImageData};

/// Pad an ImageData with constant values on each side.
///
/// Adds `pad` voxels on each side of each axis, filled with `fill_value`.
pub fn image_pad(input: &ImageData, scalars: &str, pad: [usize; 3], fill_value: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let num_components = arr.num_components();
    let origin = input.origin();
    let spacing = input.spacing();

    let new_nx = nx + 2 * pad[0];
    let new_ny = ny + 2 * pad[1];
    let new_nz = nz + 2 * pad[2];

    let new_origin = [
        origin[0] - pad[0] as f64 * spacing[0],
        origin[1] - pad[1] as f64 * spacing[1],
        origin[2] - pad[2] as f64 * spacing[2],
    ];

    let new_n = new_nx * new_ny * new_nz;
    let mut values = vec![fill_value; new_n * num_components];
    let mut buf = vec![0.0f64; num_components];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                let di = i + pad[0];
                let dj = j + pad[1];
                let dk = k + pad[2];
                let dst = (dk * new_ny * new_nx + dj * new_nx + di) * num_components;
                values[dst..dst + num_components].copy_from_slice(&buf);
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
    fn pad_1_each_side() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let values: Vec<f64> = (1..=9).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_pad(&img, "v", [1, 1, 0], 0.0);
        assert_eq!(result.dimensions(), [5, 5, 1]);

        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // Corner should be fill value
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        // Center of original data (1,1,0) -> (2,2,0) in padded = index 12
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 5.0); // center of 3x3 grid
    }

    #[test]
    fn zero_pad() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let result = image_pad(&img, "v", [0, 0, 0], 0.0);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn pads_all_components() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0, 2.0, 3.0],
                3,
            )));

        let result = image_pad(&img, "rgb", [1, 0, 0], -1.0);
        let arr = result.point_data().get_array("rgb").unwrap();
        assert_eq!(arr.num_components(), 3);

        let mut buf = [0.0; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [-1.0, -1.0, -1.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(2, 2, 1);
        let result = image_pad(&img, "nope", [1, 1, 1], 0.0);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }
}
