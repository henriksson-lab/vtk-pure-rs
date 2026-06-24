use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Local Binary Pattern (LBP) for texture classification.
///
/// For each pixel, compares with 8 neighbors. If neighbor >= center,
/// bit=1. The 8-bit pattern encodes local texture. Adds "LBP" array.
pub fn image_lbp(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let get = |i: i64, j: i64| -> f64 {
        values[(j.clamp(0, ny as i64 - 1) as usize) * nx + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    // 8 neighbors in clockwise order
    let offsets: [(i64, i64); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
    ];

    let mut lbp = vec![0.0f64; n];
    for j in 0..ny {
        for i in 0..nx {
            let center = values[j * nx + i];
            let mut pattern = 0u8;
            for (bit, &(di, dj)) in offsets.iter().enumerate() {
                if get(i as i64 + di, j as i64 + dj) >= center {
                    pattern |= 1 << bit;
                }
            }
            lbp[j * nx + i] = pattern as f64;
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("LBP", lbp, 1)));
    img
}

/// Count uniform LBP patterns (at most 2 bitwise transitions).
pub fn lbp_uniformity(input: &ImageData, scalars: &str) -> ImageData {
    let result = image_lbp(input, scalars);
    let arr = match result.point_data().get_array("LBP") {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let uniform: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let p = buf[0] as u8;
            // Count 0->1 and 1->0 transitions in circular bit pattern
            let mut transitions = 0u8;
            for bit in 0..8 {
                let cur = (p >> bit) & 1;
                let next = (p >> ((bit + 1) % 8)) & 1;
                if cur != next {
                    transitions += 1;
                }
            }
            if transitions <= 2 {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut img = result;
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LBPUniform",
            uniform,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lbp_basic() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_lbp(&img, "v");
        let arr = result.point_data().get_array("LBP").unwrap();
        assert_eq!(arr.num_tuples(), 25);
    }

    #[test]
    fn flat_all_same_pattern() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = image_lbp(&img, "v");
        let arr = result.point_data().get_array("LBP").unwrap();
        let mut buf = [0.0f64];
        // All neighbors >= center (equal) -> all bits set = 255
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 255.0);
    }

    #[test]
    fn uniform_patterns() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = lbp_uniformity(&img, "v");
        assert!(result.point_data().get_array("LBPUniform").is_some());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_lbp(&img, "nope");
        assert!(r.point_data().get_array("LBP").is_none());
    }
}
