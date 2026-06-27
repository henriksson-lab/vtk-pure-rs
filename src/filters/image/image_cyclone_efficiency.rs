//! Cyclone separation efficiency
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_cyclone_efficiency(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let particle_diameter = buf[0].abs().max(1e-12);
            let cut_diameter = 5e-6;
            particle_diameter.powi(2) / (particle_diameter.powi(2) + cut_diameter * cut_diameter)
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
        let r = image_cyclone_efficiency(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn cut_diameter_has_half_efficiency() {
        let img = ImageData::from_function(
            [1, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 5e-6,
        );
        let r = image_cyclone_efficiency(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-12);
    }
}
