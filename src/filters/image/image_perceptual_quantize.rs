//! PQ (ST.2084) encode
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_perceptual_quantize(input: &ImageData, scalars: &str) -> ImageData {
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
                let y = buf[0].max(0.0);
                let m1 = 0.1593017578125;
                let m2 = 78.84375;
                let c1 = 0.8359375;
                let c2 = 18.8515625;
                let c3 = 18.6875;
                let ym1 = y.powf(m1);
                ((c1 + c2 * ym1) / (1.0 + c3 * ym1)).powf(m2)
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
        let r = image_perceptual_quantize(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
