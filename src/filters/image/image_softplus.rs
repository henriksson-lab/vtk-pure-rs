//! Softplus activation ln(1+exp(x))

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Softplus activation ln(1+exp(x))
pub fn image_softplus(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.0 {
                buf[0] + (-buf[0]).exp().ln_1p()
            } else {
                buf[0].exp().ln_1p()
            }
        })
        .collect();
    let dims = input.dimensions();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output.set_extent(input.extent());
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_image_softplus() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_softplus(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn large_positive_input_tracks_linear_asymptote() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1000.0,
        );
        let r = image_softplus(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].is_finite());
        assert_eq!(buf[0], 1000.0);
    }
}
