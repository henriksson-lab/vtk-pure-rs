//! CFG interpolation
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_classifier_free_guidance(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if (1..=2).contains(&a.num_components()) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let components = arr.num_components();
    let mut buf = [0.0f64; 2];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let unconditional = if components > 1 { buf[0] } else { 0.0 };
            let conditional = if components > 1 { buf[1] } else { buf[0] };
            unconditional + 1.5 * (conditional - unconditional)
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
        let r = image_classifier_free_guidance(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn applies_guidance_to_unconditional_and_conditional_components() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![2.0, 6.0, 10.0, 4.0],
                2,
            )));

        let r = image_classifier_free_guidance(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 8.0);

        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn treats_single_component_as_zero_unconditional_prediction() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 4.0,
        );

        let r = image_classifier_free_guidance(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 6.0);
    }
}
