//! Unsharp mask sharpening for images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply unsharp mask: result = original + amount * (original - blurred).
///
/// In-place-style variant of [`crate::filters::image::sharpen::unsharp_mask`],
/// which holds the single implementation of the sharpening math: the result is
/// a bare image carrying only the sharpened values, stored back under the
/// `scalars` name instead of a separate "Sharpened" array.
pub fn unsharp_mask(input: &ImageData, scalars: &str, radius: usize, amount: f64) -> ImageData {
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == nx * ny * nz => {}
        _ => return input.clone(),
    }
    if dims.contains(&0) {
        return input.clone();
    }

    let sharpened = crate::filters::image::sharpen::unsharp_mask(input, scalars, radius, amount);
    let Some(arr) = sharpened.point_data().get_array("Sharpened") else {
        return input.clone();
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    ImageData::with_dimensions(nx, ny, nz)
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unsharp() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 4.0).abs() < 0.5 && (y - 4.0).abs() < 0.5 {
                    100.0
                } else {
                    0.0
                }
            },
        );
        let result = unsharp_mask(&img, "v", 1, 1.0);
        assert_eq!(result.dimensions(), [8, 8, 1]);
        // The peak should be enhanced
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(4 + 4 * 8, &mut buf);
        assert!(buf[0] > 100.0);
    }
}
