//! Clamp to 5th-95th percentile range
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_clamp_percentile(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    // Percentiles are taken over the non-NaN samples only: a NaN has no rank, and
    // letting one land on a percentile would make `clamp` panic (it requires
    // `lo <= hi`, which is false for any NaN bound).
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();
    if sorted.is_empty() {
        return input.clone();
    }
    sorted.sort_by(f64::total_cmp);
    let last = sorted.len() - 1;
    let lo = sorted[(last as f64 * 0.05) as usize];
    let hi = sorted[(last as f64 * 0.95) as usize];
    // NaNs are passed through untouched rather than clamped to a percentile they
    // were never ranked against.
    let data: Vec<f64> = values
        .into_iter()
        .map(|v| if v.is_nan() { v } else { v.clamp(lo, hi) })
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
        let r = image_clamp_percentile(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn clamps_to_data_percentiles() {
        let mut img = ImageData::with_dimensions(20, 1, 1);
        let values: Vec<f64> = (0..20).map(|i| i as f64 * 10.0).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let r = image_clamp_percentile(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(19, &mut buf);
        assert_eq!(buf[0], 180.0);
    }

    #[test]
    fn nan_values_do_not_panic() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, f64::NAN, 1.0],
                1,
            )));

        let r = image_clamp_percentile(&img, "v");
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
