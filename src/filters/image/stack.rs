use crate::data::{AnyDataArray, DataArray, ImageData};

/// Stack multiple 2D ImageData slices into a 3D volume.
///
/// All slices must have the same XY dimensions. Returns a 3D ImageData
/// with nz = number of slices.
pub fn image_stack(slices: &[&ImageData], scalars: &str) -> Option<ImageData> {
    if slices.is_empty() {
        return None;
    }
    let dims0 = slices[0].dimensions();
    let nx = dims0[0] as usize;
    let ny = dims0[1] as usize;
    let spacing = slices[0].spacing();
    let origin = slices[0].origin();
    let nz = slices.len();

    let mut values = Vec::with_capacity(nx * ny * nz);
    let mut buf = [0.0f64];

    for slice in slices {
        let arr = match slice.point_data().get_array(scalars) {
            Some(a) => a,
            None => return None,
        };
        let sd = slice.dimensions();
        if sd[0] as usize != nx || sd[1] as usize != ny {
            return None;
        }
        for i in 0..nx * ny {
            arr.tuple_as_f64(i, &mut buf);
            values.push(buf[0]);
        }
    }

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    img.set_origin(origin);
    img.set_spacing([spacing[0], spacing[1], spacing[2]]);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    Some(img)
}

/// Extract a single 2D slice from a 3D ImageData.
pub fn image_extract_slice(input: &ImageData, scalars: &str, k: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let k = k.min(nz.saturating_sub(1));
    let spacing = input.spacing();
    let origin = input.origin();

    let mut buf = [0.0f64];
    let mut values = Vec::with_capacity(nx * ny);
    for j in 0..ny {
        for i in 0..nx {
            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
            values.push(buf[0]);
        }
    }

    let mut img = ImageData::with_dimensions(nx, ny, 1);
    img.set_origin([origin[0], origin[1], origin[2] + k as f64 * spacing[2]]);
    img.set_spacing(spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_and_extract() {
        let mut s0 = ImageData::with_dimensions(3, 3, 1);
        s0.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));
        let mut s1 = ImageData::with_dimensions(3, 3, 1);
        s1.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![2.0; 9], 1)));

        let vol = image_stack(&[&s0, &s1], "v").unwrap();
        assert_eq!(vol.dimensions(), [3, 3, 2]);

        let slice = image_extract_slice(&vol, "v", 1);
        assert_eq!(slice.dimensions(), [3, 3, 1]);
        let arr = slice.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn mismatched_dims() {
        let mut s0 = ImageData::with_dimensions(3, 3, 1);
        s0.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));
        let mut s1 = ImageData::with_dimensions(4, 4, 1);
        s1.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 16],
                1,
            )));

        assert!(image_stack(&[&s0, &s1], "v").is_none());
    }

    #[test]
    fn empty_stack() {
        assert!(image_stack(&[], "v").is_none());
    }

    #[test]
    fn extract_first_slice() {
        let mut img = ImageData::with_dimensions(2, 2, 3);
        let values: Vec<f64> = (0..12).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let slice = image_extract_slice(&img, "v", 0);
        assert_eq!(slice.dimensions(), [2, 2, 1]);
    }
}
