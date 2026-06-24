use crate::data::{AnyDataArray, DataArray, ImageData};

/// Minimum intensity projection along the Z axis.
///
/// For a 3D ImageData, projects along Z by keeping the minimum scalar
/// value at each (x, y) position.  Returns a 2D ImageData with nz=1.
pub fn min_projection_z(input: &ImageData, scalars: &str) -> ImageData {
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

    let out_len: usize = nx * ny;
    let mut values: Vec<f64> = vec![f64::INFINITY; out_len];
    let mut buf = [0.0f64];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let src_idx: usize = k * ny * nx + j * nx + i;
                let dst_idx: usize = j * nx + i;
                arr.tuple_as_f64(src_idx, &mut buf);
                if buf[0] < values[dst_idx] {
                    values[dst_idx] = buf[0];
                }
            }
        }
    }

    let mut out = ImageData::with_dimensions(nx, ny, 1);
    let spacing = input.spacing();
    out.set_spacing([spacing[0], spacing[1], 1.0]);
    let origin = input.origin();
    out.set_origin(origin);
    out.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_min_projection() {
        // 2x2x3 image
        let mut img = ImageData::with_dimensions(2, 2, 3);
        let mut vals: Vec<f64> = vec![100.0; 12];
        // z=0 layer
        vals[0] = 5.0;
        vals[1] = 10.0;
        vals[2] = 15.0;
        vals[3] = 20.0;
        // z=1 layer
        vals[4] = 1.0;
        vals[5] = 50.0;
        vals[6] = 50.0;
        vals[7] = 50.0;
        // z=2 layer
        vals[8] = 50.0;
        vals[9] = 2.0;
        vals[10] = 50.0;
        vals[11] = 50.0;

        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));

        let result = min_projection_z(&img, "s");
        let dims = result.dimensions();
        assert_eq!(dims[0], 2);
        assert_eq!(dims[1], 2);
        assert_eq!(dims[2], 1);

        let arr = result.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 2.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 15.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 20.0);
    }

    #[test]
    fn single_slice_unchanged() {
        let mut img = ImageData::with_dimensions(3, 2, 1);
        let vals: Vec<f64> = vec![5.0, 3.0, 1.0, 2.0, 4.0, 6.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", vals, 1)));

        let result = min_projection_z(&img, "data");
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
        let result = min_projection_z(&img, "nonexistent");
        assert_eq!(result.dimensions(), [1, 1, 1]);
    }
}
