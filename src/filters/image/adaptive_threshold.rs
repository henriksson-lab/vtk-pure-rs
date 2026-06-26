//! Adaptive thresholding using local mean.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Adaptive threshold: pixel = 1 if value > local_mean - offset, else 0.
pub fn adaptive_threshold(
    input: &ImageData,
    scalars: &str,
    radius: usize,
    offset: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let r = radius as isize;
    let (nx, ny) = (dims[0], dims[1]);

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iz = idx / (nx * ny);
            let rem = idx % (nx * ny);
            let iy = rem / nx;
            let ix = rem % nx;
            let mut sum = 0.0;
            let mut count = 0.0;
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                        sum += vals[sx as usize + sy as usize * nx + iz * nx * ny];
                        count += 1.0;
                    }
                }
            }
            let local_mean = sum / count;
            if vals[idx] > local_mean - offset {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_adaptive() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if x > 3.0 && x < 7.0 && y > 3.0 && y < 7.0 {
                    200.0
                } else {
                    50.0
                }
            },
        );
        let result = adaptive_threshold(&img, "v", 2, 0.0);
        assert_eq!(result.dimensions(), [10, 10, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5 + 5 * 10, &mut buf);
        assert_eq!(buf[0], 1.0); // center bright pixel
    }
}
