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

/// Compute the dot product of two vector arrays on ImageData.
///
/// Both arrays must have the same number of components.
/// Adds a scalar "DotProduct" array.
pub fn image_dot_product(input: &ImageData, a_name: &str, b_name: &str) -> ImageData {
    let aa = match input.point_data().get_array(a_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let ba = match input.point_data().get_array(b_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let nc = aa.num_components();
    if nc != ba.num_components() {
        return input.clone();
    }
    if aa.scalar_type() != ba.scalar_type() {
        return input.clone();
    }
    if aa.num_tuples() != ba.num_tuples() {
        return input.clone();
    }

    let n = aa.num_tuples();
    let mut abuf = vec![0.0f64; nc];
    let mut bbuf = vec![0.0f64; nc];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            aa.tuple_as_f64(i, &mut abuf);
            ba.tuple_as_f64(i, &mut bbuf);
            (0..nc).map(|c| abuf[c] * bbuf[c]).sum()
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut().add_array(array_from_f64_values(
        "DotProduct",
        values,
        1,
        aa.scalar_type(),
    ));
    img
}

/// Compute the cross product of two 3-component vector arrays.
pub fn image_cross_product(input: &ImageData, a_name: &str, b_name: &str) -> ImageData {
    let aa = match input.point_data().get_array(a_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let ba = match input.point_data().get_array(b_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    if aa.num_components() != 3 || ba.num_components() != 3 {
        return input.clone();
    }
    if aa.scalar_type() != ba.scalar_type() {
        return input.clone();
    }
    if aa.num_tuples() != ba.num_tuples() {
        return input.clone();
    }

    let n = aa.num_tuples();
    let mut ab = [0.0f64; 3];
    let mut bb = [0.0f64; 3];
    let mut values = Vec::with_capacity(n * 3);
    for i in 0..n {
        aa.tuple_as_f64(i, &mut ab);
        ba.tuple_as_f64(i, &mut bb);
        values.push(ab[1] * bb[2] - ab[2] * bb[1]);
        values.push(ab[2] * bb[0] - ab[0] * bb[2]);
        values.push(ab[0] * bb[1] - ab[1] * bb[0]);
    }

    let mut img = input.clone();
    img.point_data_mut().add_array(array_from_f64_values(
        "CrossProduct",
        values,
        3,
        aa.scalar_type(),
    ));
    img
}

/// Scale a vector array by a scalar array element-wise.
pub fn image_scale_vector(
    input: &ImageData,
    vec_name: &str,
    scalar_name: &str,
    output: &str,
) -> ImageData {
    let va = match input.point_data().get_array(vec_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let sa = match input.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let nc = va.num_components();
    if va.num_tuples() != sa.num_tuples() {
        return input.clone();
    }
    let n = va.num_tuples();

    let mut vbuf = vec![0.0f64; nc];
    let mut sbuf = [0.0f64];
    let mut values = Vec::with_capacity(n * nc);
    for i in 0..n {
        va.tuple_as_f64(i, &mut vbuf);
        sa.tuple_as_f64(i, &mut sbuf);
        for c in 0..nc {
            values.push(vbuf[c] * sbuf[0]);
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(array_from_f64_values(output, values, nc, va.scalar_type()));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_product() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![0.0, 1.0, 0.0],
                3,
            )));

        let result = image_dot_product(&img, "a", "b");
        let arr = result.point_data().get_array("DotProduct").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // perpendicular
    }

    #[test]
    fn cross_product() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![0.0, 1.0, 0.0],
                3,
            )));

        let result = image_cross_product(&img, "a", "b");
        let arr = result.point_data().get_array("CrossProduct").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 1.0]); // x cross y = z
    }

    #[test]
    fn scale_vector() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                3,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![2.0, 0.5],
                1,
            )));

        let result = image_scale_vector(&img, "v", "s", "out");
        let arr = result.point_data().get_array("out").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [2.0, 4.0, 6.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [2.0, 2.5, 3.0]);
    }

    #[test]
    fn missing_arrays() {
        let img = ImageData::with_dimensions(1, 1, 1);
        let r = image_dot_product(&img, "a", "b");
        assert!(r.point_data().get_array("DotProduct").is_none());
    }

    #[test]
    fn dot_product_preserves_scalar_type() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::I16(DataArray::from_vec(
                "a",
                vec![2i16, 3, 4],
                3,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::I16(DataArray::from_vec(
                "b",
                vec![5i16, 6, 7],
                3,
            )));

        let result = image_dot_product(&img, "a", "b");
        let arr = result.point_data().get_array("DotProduct").unwrap();
        assert_eq!(arr.scalar_type(), ScalarType::I16);
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 56.0);
    }

    #[test]
    fn scalar_type_mismatch_returns_clone() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                "b",
                vec![0.0f32, 1.0, 0.0],
                3,
            )));

        let result = image_dot_product(&img, "a", "b");
        assert!(result.point_data().get_array("DotProduct").is_none());
    }

    #[test]
    fn tuple_count_mismatch_returns_clone() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                3,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![1.0, 2.0, 3.0],
                3,
            )));

        let dot = image_dot_product(&img, "a", "b");
        assert!(dot.point_data().get_array("DotProduct").is_none());
        let cross = image_cross_product(&img, "a", "b");
        assert!(cross.point_data().get_array("CrossProduct").is_none());
        let scale = image_scale_vector(&img, "a", "b", "out");
        assert!(scale.point_data().get_array("out").is_none());
    }
}
