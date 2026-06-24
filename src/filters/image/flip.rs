use crate::data::{AnyDataArray, DataArray, ImageData};

/// Flip an ImageData along one or more axes.
///
/// Reverses the order of voxels along the specified axes.
pub fn image_flip(
    input: &ImageData,
    scalars: &str,
    flip_x: bool,
    flip_y: bool,
    flip_z: bool,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let si = if flip_x { nx - 1 - i } else { i };
                let sj = if flip_y { ny - 1 - j } else { j };
                let sk = if flip_z { nz - 1 - k } else { k };
                let src_idx = sk * ny * nx + sj * nx + si;
                let dst_idx = k * ny * nx + j * nx + i;
                arr.tuple_as_f64(src_idx, &mut buf);
                values[dst_idx] = buf[0];
            }
        }
    }

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_x() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let result = image_flip(&img, "v", true, false, false);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 3.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn flip_y() {
        let mut img = ImageData::with_dimensions(1, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let result = image_flip(&img, "v", false, true, false);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 30.0);
    }

    #[test]
    fn no_flip() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));
        let result = image_flip(&img, "v", false, false, false);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
