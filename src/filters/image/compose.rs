use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compose 3 scalar arrays into a single 3-component vector array.
pub fn image_compose_vector(
    input: &ImageData,
    x_name: &str,
    y_name: &str,
    z_name: &str,
    output: &str,
) -> ImageData {
    let ax = match input.point_data().get_array(x_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let ay = match input.point_data().get_array(y_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let az = match input.point_data().get_array(z_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = ax.num_tuples().min(ay.num_tuples()).min(az.num_tuples());
    let mut bx = [0.0f64];
    let mut by = [0.0f64];
    let mut bz = [0.0f64];
    let mut values = Vec::with_capacity(n * 3);
    for i in 0..n {
        ax.tuple_as_f64(i, &mut bx);
        ay.tuple_as_f64(i, &mut by);
        az.tuple_as_f64(i, &mut bz);
        values.push(bx[0]);
        values.push(by[0]);
        values.push(bz[0]);
    }
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, 3)));
    img
}

/// Decompose a 3-component vector array into 3 scalar arrays.
pub fn image_decompose_vector(
    input: &ImageData,
    vec_name: &str,
    x_out: &str,
    y_out: &str,
    z_out: &str,
) -> ImageData {
    let arr = match input.point_data().get_array(vec_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    if arr.num_components() != 3 {
        return input.clone();
    }
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut zs = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        xs.push(buf[0]);
        ys.push(buf[1]);
        zs.push(buf[2]);
    }
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(x_out, xs, 1)));
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(y_out, ys, 1)));
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(z_out, zs, 1)));
    img
}

/// Compute the magnitude of a vector array on ImageData.
pub fn image_vector_magnitude(input: &ImageData, vec_name: &str, output: &str) -> ImageData {
    let arr = match input.point_data().get_array(vec_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let nc = arr.num_components();
    let n = arr.num_tuples();
    let mut buf = vec![0.0f64; nc];
    let mut mags = Vec::with_capacity(n);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let m: f64 = buf.iter().map(|v| v * v).sum::<f64>().sqrt();
        mags.push(m);
    }
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, mags, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_decompose() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vx",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vy",
                vec![4.0, 5.0, 6.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vz",
                vec![7.0, 8.0, 9.0],
                1,
            )));

        let composed = image_compose_vector(&img, "vx", "vy", "vz", "vel");
        let arr = composed.point_data().get_array("vel").unwrap();
        assert_eq!(arr.num_components(), 3);

        let decomposed = image_decompose_vector(&composed, "vel", "a", "b", "c");
        let ax = decomposed.point_data().get_array("a").unwrap();
        let mut buf = [0.0f64];
        ax.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn vector_magnitude() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0, 0.0],
                3,
            )));
        let result = image_vector_magnitude(&img, "v", "mag");
        let arr = result.point_data().get_array("mag").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(2, 1, 1);
        let r = image_compose_vector(&img, "a", "b", "c", "out");
        assert!(r.point_data().get_array("out").is_none());
    }
}
