use crate::data::{AnyDataArray, DataArray, ImageData};

/// Sum intensity projection along the Z axis.
///
/// For a 3D ImageData, sums the scalar values at each (x, y) position over all
/// Z slices. Returns a 2D ImageData with nz=1.
pub fn sum_projection_z(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => {
            return ImageData::with_dimensions(1, 1, 1);
        }
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let num_components = arr.num_components();

    let out_len: usize = nx * ny;
    let mut values: Vec<f64> = vec![0.0; out_len * num_components];
    let mut buf = vec![0.0f64; num_components];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let src_idx: usize = k * ny * nx + j * nx + i;
                let dst_idx: usize = j * nx + i;
                arr.tuple_as_f64(src_idx, &mut buf);
                let dst_offset = dst_idx * num_components;
                for component in 0..num_components {
                    values[dst_offset + component] += buf[component];
                }
            }
        }
    }

    let mut out = ImageData::with_dimensions(nx, ny, 1);
    let spacing = input.spacing();
    out.set_spacing(spacing);
    let mut origin = input.origin();
    origin[2] += 0.5 * spacing[2] * (nz.saturating_sub(1) as f64);
    out.set_origin(origin);
    out.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            values,
            num_components,
        )));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_sum_projection() {
        // 2x2x3 image
        let mut img = ImageData::with_dimensions(2, 2, 3);
        let mut vals: Vec<f64> = vec![0.0; 12];
        // z=0 layer
        vals[0] = 1.0;
        vals[1] = 2.0;
        vals[2] = 3.0;
        vals[3] = 4.0;
        // z=1 layer
        vals[4] = 10.0;
        vals[5] = 20.0;
        vals[6] = 30.0;
        vals[7] = 40.0;
        // z=2 layer
        vals[8] = 100.0;
        vals[9] = 200.0;
        vals[10] = 300.0;
        vals[11] = 400.0;

        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));

        let result = sum_projection_z(&img, "s");
        let dims = result.dimensions();
        assert_eq!(dims[0], 2);
        assert_eq!(dims[1], 2);
        assert_eq!(dims[2], 1);

        let arr = result.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 111.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 222.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 333.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 444.0);
    }

    #[test]
    fn single_slice_unchanged() {
        let mut img = ImageData::with_dimensions(3, 2, 1);
        let vals: Vec<f64> = vec![5.0, 3.0, 1.0, 2.0, 4.0, 6.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", vals, 1)));

        let result = sum_projection_z(&img, "data");
        assert_eq!(result.dimensions()[2], 1);
        let arr = result.point_data().get_array("data").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0);
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 6.0);
    }

    #[test]
    fn missing_array_returns_default() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = sum_projection_z(&img, "nonexistent");
        assert_eq!(result.dimensions(), [1, 1, 1]);
    }

    #[test]
    fn processes_all_components_and_centers_origin() {
        let mut img = ImageData::with_dimensions(1, 1, 3);
        img.set_spacing([1.0, 2.0, 4.0]);
        img.set_origin([10.0, 20.0, 30.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![
                    1.0, 2.0, //
                    3.0, 4.0, //
                    5.0, 6.0,
                ],
                2,
            )));

        let result = sum_projection_z(&img, "s");
        assert_eq!(result.spacing(), [1.0, 2.0, 4.0]);
        assert_eq!(result.origin(), [10.0, 20.0, 34.0]);
        let arr = result.point_data().get_array("s").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [9.0, 12.0]);
    }
}
