use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute integral image (summed area table) for fast box queries.
///
/// After computing, the sum of any rectangular region can be found
/// in O(1) using: sum = SAT[r2][c2] - SAT[r1-1][c2] - SAT[r2][c1-1] + SAT[r1-1][c1-1].
/// Adds "IntegralImage" array.
pub fn integral_image_2d(input: &ImageData, scalars: &str) -> ImageData {
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

    let mut sat = vec![0.0f64; n];
    for j in 0..ny {
        for i in 0..nx {
            let v = values[j * nx + i];
            let left = if i > 0 { sat[j * nx + i - 1] } else { 0.0 };
            let top = if j > 0 { sat[(j - 1) * nx + i] } else { 0.0 };
            let diag = if i > 0 && j > 0 {
                sat[(j - 1) * nx + i - 1]
            } else {
                0.0
            };
            sat[j * nx + i] = v + left + top - diag;
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IntegralImage",
            sat,
            1,
        )));
    img
}

/// Query a rectangular sum from a precomputed integral image.
pub fn rect_sum(sat: &[f64], nx: usize, r1: usize, c1: usize, r2: usize, c2: usize) -> f64 {
    let a = sat[r2 * nx + c2];
    let b = if r1 > 0 { sat[(r1 - 1) * nx + c2] } else { 0.0 };
    let c = if c1 > 0 { sat[r2 * nx + c1 - 1] } else { 0.0 };
    let d = if r1 > 0 && c1 > 0 {
        sat[(r1 - 1) * nx + c1 - 1]
    } else {
        0.0
    };
    a - b - c + d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integral_basic() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = integral_image_2d(&img, "v");
        let arr = result.point_data().get_array("IntegralImage").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(8, &mut buf); // bottom-right = sum of all
        assert_eq!(buf[0], 9.0);
    }

    #[test]
    fn rect_query() {
        let sat = vec![1.0, 3.0, 6.0, 5.0, 12.0, 21.0, 12.0, 27.0, 45.0]; // from [1,2,3,4,5,6,7,8,9]
                                                                          // Sum of bottom-right 2x2: 5+6+8+9 = 28
        let s = rect_sum(&sat, 3, 1, 1, 2, 2);
        assert_eq!(s, 28.0);
    }

    #[test]
    fn full_sum() {
        let sat = vec![1.0, 3.0, 6.0, 5.0, 12.0, 21.0, 12.0, 27.0, 45.0];
        let s = rect_sum(&sat, 3, 0, 0, 2, 2);
        assert_eq!(s, 45.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = integral_image_2d(&img, "nope");
        assert!(r.point_data().get_array("IntegralImage").is_none());
    }
}
