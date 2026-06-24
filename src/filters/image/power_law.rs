use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply gamma/power-law correction to an ImageData scalar field.
///
/// output = (input / max)^gamma * max. Gamma < 1 brightens, > 1 darkens.
pub fn image_gamma(input: &ImageData, scalars: &str, gamma: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];

    let mut max_v = 0.0f64;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        max_v = max_v.max(buf[0]);
    }
    if max_v < 1e-15 {
        return input.clone();
    }

    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0].max(0.0) / max_v).powf(gamma) * max_v
        })
        .collect();

    let mut img = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                1,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = attrs;
    img
}

/// Apply sigmoid contrast enhancement: output = 1/(1+exp(-gain*(input-midpoint))).
pub fn image_sigmoid(input: &ImageData, scalars: &str, gain: f64, midpoint: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];

    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            1.0 / (1.0 + (-gain * (buf[0] - midpoint)).exp())
        })
        .collect();

    let mut img = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                1,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_1_identity() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 50.0, 100.0],
                1,
            )));
        let result = image_gamma(&img, "v", 1.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 100.0).abs() < 1e-10);
    }

    #[test]
    fn gamma_brightens() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 50.0, 100.0],
                1,
            )));
        let bright = image_gamma(&img, "v", 0.5);
        let arr = bright.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 50.0, "midtone={}", buf[0]); // gamma<1 brightens midtones
    }

    #[test]
    fn sigmoid_basic() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.5, 1.0],
                1,
            )));
        let result = image_sigmoid(&img, "v", 10.0, 0.5);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.5).abs() < 0.01); // midpoint -> 0.5
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_gamma(&img, "nope", 2.0);
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
