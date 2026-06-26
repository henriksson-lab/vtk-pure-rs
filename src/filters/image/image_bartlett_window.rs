//! Bartlett (triangle) window
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_bartlett_window(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let data: Vec<f64> = if n > 1 {
        let denominator = (n - 1) as f64;
        (0..n)
            .map(|i| {
                let x = i.min(n - 1 - i) as f64;
                2.0 * x / denominator
            })
            .collect()
    } else {
        vec![0.0; n]
    };
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_bartlett_window(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn matches_vtk_bartlett_kernel() {
        let img = ImageData::from_function(
            [10, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        let r = image_bartlett_window(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let expected = [
            0.0,
            0.2222222222222222,
            0.4444444444444444,
            0.6666666666666666,
            0.8888888888888888,
            0.8888888888888888,
            0.6666666666666666,
            0.4444444444444444,
            0.2222222222222222,
            0.0,
        ];
        let mut buf = [0.0f64];
        for (i, expected) in expected.iter().enumerate() {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - expected).abs() < 1e-12);
        }
    }
}
