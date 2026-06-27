//! Diffusion tensor trace
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_diffusion_tensor(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if matches!(a.num_components(), 1 | 6 | 9) => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let components = arr.num_components();
    let mut buf = [0.0f64; 9];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            match components {
                1 => buf[0] * 3.0,
                6 => buf[0] + buf[1] + buf[2],
                9 => buf[0] + buf[4] + buf[8],
                _ => unreachable!(),
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

    fn image(values: Vec<f64>, components: usize) -> ImageData {
        ImageData::with_dimensions(values.len() / components, 1, 1)
            .with_spacing([0.5, 2.0, 1.0])
            .with_origin([1.0, -1.0, 0.0])
            .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                "v", values, components,
            )))
    }

    #[test]
    fn computes_isotropic_scalar_trace() {
        let img = image(vec![2.0, -1.0], 1);
        let r = image_diffusion_tensor(&img, "v");
        assert_eq!(r.dimensions(), [2, 1, 1]);
        assert_eq!(r.spacing(), img.spacing());
        assert_eq!(r.origin(), img.origin());
        assert_eq!(
            r.point_data().get_array("v").unwrap().to_f64_vec(),
            vec![6.0, -3.0]
        );
    }

    #[test]
    fn computes_symmetric_tensor_trace() {
        let img = image(vec![1.0, 2.0, 3.0, 10.0, 20.0, 30.0], 6);
        let r = image_diffusion_tensor(&img, "v");
        assert_eq!(
            r.point_data().get_array("v").unwrap().to_f64_vec(),
            vec![6.0]
        );
    }

    #[test]
    fn computes_full_tensor_trace() {
        let img = image(vec![1.0, 10.0, 20.0, 30.0, 2.0, 40.0, 50.0, 60.0, 3.0], 9);
        let r = image_diffusion_tensor(&img, "v");
        assert_eq!(
            r.point_data().get_array("v").unwrap().to_f64_vec(),
            vec![6.0]
        );
    }
}
