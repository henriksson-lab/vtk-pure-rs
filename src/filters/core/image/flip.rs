use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::{Scalar, ScalarType};

fn array_from_f64_values(
    name: &str,
    values: Vec<f64>,
    num_components: usize,
    scalar_type: ScalarType,
) -> AnyDataArray {
    fn cast_array<T: Scalar>(name: &str, values: Vec<f64>, num_components: usize) -> AnyDataArray
    where
        AnyDataArray: From<DataArray<T>>,
    {
        AnyDataArray::from(DataArray::from_vec(
            name,
            values.into_iter().map(T::from_f64).collect(),
            num_components,
        ))
    }

    match scalar_type {
        ScalarType::F32 => cast_array::<f32>(name, values, num_components),
        ScalarType::F64 => cast_array::<f64>(name, values, num_components),
        ScalarType::I8 => cast_array::<i8>(name, values, num_components),
        ScalarType::I16 => cast_array::<i16>(name, values, num_components),
        ScalarType::I32 => cast_array::<i32>(name, values, num_components),
        ScalarType::I64 => cast_array::<i64>(name, values, num_components),
        ScalarType::U8 => cast_array::<u8>(name, values, num_components),
        ScalarType::U16 => cast_array::<u16>(name, values, num_components),
        ScalarType::U32 => cast_array::<u32>(name, values, num_components),
        ScalarType::U64 => cast_array::<u64>(name, values, num_components),
    }
}

fn flip_array(
    arr: &AnyDataArray,
    dims: [usize; 3],
    flip_x: bool,
    flip_y: bool,
    flip_z: bool,
) -> Option<AnyDataArray> {
    let [nx, ny, nz] = dims;
    let n = nx * ny * nz;
    let nc = arr.num_components();
    if arr.num_tuples() != n {
        return None;
    }

    let mut values = vec![0.0f64; n * nc];
    let mut buf = vec![0.0f64; nc];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let si = if flip_x { nx - 1 - i } else { i };
                let sj = if flip_y { ny - 1 - j } else { j };
                let sk = if flip_z { nz - 1 - k } else { k };
                let src_idx = sk * ny * nx + sj * nx + si;
                let dst_idx = k * ny * nx + j * nx + i;
                arr.tuple_as_f64(src_idx, &mut buf);
                values[dst_idx * nc..(dst_idx + 1) * nc].copy_from_slice(&buf);
            }
        }
    }

    Some(array_from_f64_values(
        arr.name(),
        values,
        nc,
        arr.scalar_type(),
    ))
}

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
    if arr.num_tuples() != n {
        return input.clone();
    }

    let mut img = input.clone();
    let mut new_attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        new_attrs.add_array(
            flip_array(a, [nx, ny, nz], flip_x, flip_y, flip_z).unwrap_or_else(|| a.clone()),
        );
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

    #[test]
    fn flip_preserves_components_and_type() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "rgb",
                vec![1, 2, 3, 4, 5, 6],
                3,
            )));
        let result = image_flip(&img, "rgb", true, false, false);
        let arr = result.point_data().get_array("rgb").unwrap();
        assert_eq!(arr.num_components(), 3);
        assert_eq!(arr.scalar_type(), crate::types::ScalarType::I32);
        assert_eq!(arr.to_f64_vec_flat(), vec![4.0, 5.0, 6.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn flip_reindexes_all_image_sized_arrays() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "selected",
                vec![1.0, 2.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "other",
                vec![10.0, 20.0],
                1,
            )));

        let result = image_flip(&img, "selected", true, false, false);
        assert_eq!(
            result
                .point_data()
                .get_array("selected")
                .unwrap()
                .to_f64_vec(),
            vec![2.0, 1.0]
        );
        assert_eq!(
            result.point_data().get_array("other").unwrap().to_f64_vec(),
            vec![20.0, 10.0]
        );
    }
}
