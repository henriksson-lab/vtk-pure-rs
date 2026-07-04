//! Predicted mean vote (simplified)
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_pmv_thermal_comfort(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (0.303 * (-0.036 * buf[0]).exp() + 0.028) * (buf[0] - 58.15)
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
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_pmv_thermal_comfort(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn applies_pmv_metabolic_weight_to_heat_load() {
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![58.15, 100.0], 1),
        ));

        let r = image_pmv_thermal_comfort(&img, "v");
        let values = r.point_data().get_array("v").unwrap().to_f64_vec();

        assert_eq!(values[0], 0.0);
        let expected = (0.303 * (-0.036f64 * 100.0).exp() + 0.028) * (100.0 - 58.15);
        assert!((values[1] - expected).abs() < 1e-12);
    }
}
