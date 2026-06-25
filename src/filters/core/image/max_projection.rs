use crate::data::{AnyDataArray, DataArray, ImageData};

/// Maximum intensity projection along the Z axis.
///
/// For a 3D ImageData, projects along Z by keeping the maximum scalar
/// value at each (x, y) position. Returns a 2D ImageData with nz=1.
pub fn max_projection_z(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => {
            // No such array; return a 1x1x1 empty image
            return ImageData::with_dimensions(1, 1, 1);
        }
    };

    let dims = input.dimensions();
    let num_components = arr.num_components();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;

    let out_len: usize = nx * ny;
    let mut values: Vec<f64> = vec![f64::NEG_INFINITY; out_len * num_components];
    let mut buf = vec![0.0f64; num_components];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let src_idx: usize = k * ny * nx + j * nx + i;
                let dst_idx: usize = j * nx + i;
                arr.tuple_as_f64(src_idx, &mut buf);
                for c in 0..num_components {
                    let component_idx = dst_idx * num_components + c;
                    if buf[c] > values[component_idx] {
                        values[component_idx] = buf[c];
                    }
                }
            }
        }
    }

    let mut out = ImageData::with_dimensions(nx, ny, 1);
    let spacing = input.spacing();
    out.set_spacing([spacing[0], spacing[1], 1.0]);
    let mut origin = input.origin();
    origin[2] += 0.5 * spacing[2] * (nz.saturating_sub(1)) as f64;
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
    fn simple_max_projection() {
        // 2x2x3 image: at each (x,y), the max over z should be found
        let mut img = ImageData::with_dimensions(2, 2, 3);
        let mut vals: Vec<f64> = vec![0.0; 12];
        // z=0 layer
        vals[0] = 1.0;
        vals[1] = 2.0;
        vals[2] = 3.0;
        vals[3] = 4.0;
        // z=1 layer
        vals[4] = 10.0;
        vals[5] = 0.0;
        vals[6] = 0.0;
        vals[7] = 0.0;
        // z=2 layer
        vals[8] = 0.0;
        vals[9] = 20.0;
        vals[10] = 0.0;
        vals[11] = 0.0;

        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));

        let result = max_projection_z(&img, "s");
        let dims = result.dimensions();
        assert_eq!(dims[0], 2);
        assert_eq!(dims[1], 2);
        assert_eq!(dims[2], 1);

        let arr = result.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 20.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 3.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 4.0);
    }

    #[test]
    fn single_slice() {
        let mut img = ImageData::with_dimensions(3, 2, 1);
        let vals: Vec<f64> = vec![5.0, 3.0, 1.0, 2.0, 4.0, 6.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", vals, 1)));

        let result = max_projection_z(&img, "data");
        assert_eq!(result.dimensions()[2], 1);
        let arr = result.point_data().get_array("data").unwrap();
        let mut buf = [0.0f64];
        // Values unchanged since nz=1
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0);
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 6.0);
    }

    #[test]
    fn missing_array_returns_default() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = max_projection_z(&img, "nonexistent");
        assert_eq!(result.dimensions(), [1, 1, 1]);
    }
}
