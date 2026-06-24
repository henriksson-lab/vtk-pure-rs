use crate::data::{AnyDataArray, DataArray, ImageData};

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

    let n = aa.num_tuples().min(ba.num_tuples());
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
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DotProduct",
            values,
            1,
        )));
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

    let n = aa.num_tuples().min(ba.num_tuples());
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
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CrossProduct",
            values,
            3,
        )));
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
    let n = va.num_tuples().min(sa.num_tuples());

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
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, nc)));
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
}
