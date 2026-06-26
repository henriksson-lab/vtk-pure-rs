use crate::data::{AnyDataArray, DataArray, ImageData};

fn image_point_count(dims: [usize; 3]) -> Option<usize> {
    dims[0].checked_mul(dims[1])?.checked_mul(dims[2])
}

/// Remap scalar values from their current [old_min, old_max] range to [new_min, new_max].
///
/// Reads the named scalar array, finds its min/max, then applies a linear mapping
/// to produce a new "Remapped" array added to point data.
pub fn remap_range(input: &ImageData, scalars: &str, new_min: f64, new_max: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    let image_points = match image_point_count(input.dimensions()) {
        Some(n) => n,
        None => return input.clone(),
    };
    if n == 0 || n < image_points {
        return input.clone();
    }

    // Find current min/max
    let mut buf = [0.0f64];
    let mut old_min: f64 = f64::MAX;
    let mut old_max: f64 = f64::MIN;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v: f64 = buf[0];
        if v < old_min {
            old_min = v;
        }
        if v > old_max {
            old_max = v;
        }
    }

    let old_range: f64 = old_max - old_min;
    let new_range: f64 = new_max - new_min;

    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if old_range.abs() < 1e-15 {
                // All values are the same; map to midpoint of new range
                (new_min + new_max) * 0.5
            } else {
                let t: f64 = (buf[0] - old_min) / old_range;
                new_min + t * new_range
            }
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Remapped", values, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remap_0_100_to_0_1() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                1,
            )));

        let result = remap_range(&img, "val", 0.0, 1.0);
        let arr = result.point_data().get_array("Remapped").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn remap_inverted_range() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 5.0, 10.0],
                1,
            )));

        let result = remap_range(&img, "val", 10.0, 0.0);
        let arr = result.point_data().get_array("Remapped").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 10.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = remap_range(&img, "nope", 0.0, 1.0);
        assert_eq!(result.dimensions(), [3, 1, 1]);
    }
}
