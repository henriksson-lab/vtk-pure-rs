//! Isostatic compensation (Airy model)
use crate::data::{AnyDataArray, DataArray, ImageData};

const CRUST_DENSITY: f64 = 2670.0;
const MANTLE_DENSITY: f64 = 3300.0;

fn airy_root_depth(topography: f64) -> f64 {
    topography * CRUST_DENSITY / (MANTLE_DENSITY - CRUST_DENSITY)
}

pub fn image_isostatic_compensation(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            airy_root_depth(buf[0])
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
        let r = image_isostatic_compensation(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut out = [0.0f64];
        arr.tuple_as_f64(0, &mut out);
        assert!((out[0] - airy_root_depth(1.0)).abs() < 1e-12);
    }

    #[test]
    fn preserves_input_extent() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_extent([5, 7, 10, 12, 2, 2]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let r = image_isostatic_compensation(&img, "v");
        assert_eq!(r.extent(), [5, 7, 10, 12, 2, 2]);
    }
}
