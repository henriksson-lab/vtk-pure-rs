//! Mean shift filtering for image segmentation.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply mean shift filtering (spatial + range filtering).
pub fn mean_shift_filter(
    input: &ImageData,
    scalars: &str,
    spatial_radius: usize,
    range_radius: f64,
    iterations: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let sr = spatial_radius as isize;

    for _ in 0..iterations {
        let prev = vals.clone();
        for idx in 0..n {
            let iy = idx / nx;
            let ix = idx % nx;
            let center = prev[idx];
            let mut sum = 0.0;
            let mut count = 0.0;
            for dy in -sr..=sr {
                for dx in -sr..=sr {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx < 0 || sx >= nx as isize || sy < 0 || sy >= ny as isize {
                        continue;
                    }
                    let v = prev[sx as usize + sy as usize * nx];
                    if (v - center).abs() <= range_radius {
                        sum += v;
                        count += 1.0;
                    }
                }
            }
            vals[idx] = if count > 0.0 { sum / count } else { center };
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, vals, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mean_shift() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| if x < 5.0 { 10.0 } else { 90.0 },
        );
        let r = mean_shift_filter(&img, "v", 2, 20.0, 3);
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
}
