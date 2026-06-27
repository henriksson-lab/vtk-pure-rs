//! Ease-in bounce approximation
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_ease_in_bounce(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            {
                let t = 1.0 - buf[0].clamp(0.0, 1.0);
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    1.0 - n1 * t * t
                } else if t < 2.0 / d1 {
                    1.0 - n1 * (t - 1.5 / d1) * (t - 1.5 / d1) - 0.75
                } else if t < 2.5 / d1 {
                    1.0 - n1 * (t - 2.25 / d1) * (t - 2.25 / d1) - 0.9375
                } else {
                    1.0 - n1 * (t - 2.625 / d1) * (t - 2.625 / d1) - 0.984375
                }
            }
        })
        .collect();
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
        let r = image_ease_in_bounce(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn endpoints_match_ease_in_bounce() {
        let img = ImageData::from_function(
            [2, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = image_ease_in_bounce(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
