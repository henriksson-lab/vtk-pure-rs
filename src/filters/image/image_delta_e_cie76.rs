//! CIE76 color difference
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_delta_e_cie76(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0].powi(2) + buf[1].powi(2) + buf[2].powi(2)).sqrt()
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
        let img = ImageData::with_dimensions(2, 1, 1).with_point_array(AnyDataArray::F64(
            DataArray::from_vec("v", vec![3.0, 4.0, 12.0, 0.0, 5.0, 12.0], 3),
        ));
        let r = image_delta_e_cie76(&img, "v");
        assert_eq!(r.dimensions(), [2, 1, 1]);

        let arr = r.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 1);
        assert_eq!(arr.to_f64_vec(), vec![13.0, 13.0]);
    }
}
