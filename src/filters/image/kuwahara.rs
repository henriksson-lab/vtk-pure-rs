//! Kuwahara filter for painterly edge-preserving smoothing.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply Kuwahara filter. Selects the sub-region with lowest variance.
pub fn kuwahara_filter(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let n = nx.saturating_mul(ny).saturating_mul(nz);
    if n == 0 || arr.num_tuples() < n {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let r = radius as isize;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let slice_size = nx * ny;
            let slice_offset = idx / slice_size * slice_size;
            let in_slice = idx - slice_offset;
            let iy = (in_slice / nx) as isize;
            let ix = (in_slice % nx) as isize;
            let quadrants: [(isize, isize, isize, isize); 4] = [
                (ix - r, ix, iy - r, iy),
                (ix, ix + r, iy - r, iy),
                (ix - r, ix, iy, iy + r),
                (ix, ix + r, iy, iy + r),
            ];
            let mut best_mean = 0.0;
            let mut best_var = f64::INFINITY;
            for &(x0, x1, y0, y1) in &quadrants {
                let (mean, var) = region_stats(
                    &vals,
                    slice_offset,
                    x0,
                    x1,
                    y0,
                    y1,
                    nx as isize,
                    ny as isize,
                );
                if var < best_var {
                    best_var = var;
                    best_mean = mean;
                }
            }
            best_mean
        })
        .collect();

    ImageData::with_dimensions(nx, ny, nz)
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

fn region_stats(
    vals: &[f64],
    slice_offset: usize,
    x0: isize,
    x1: isize,
    y0: isize,
    y1: isize,
    nx: isize,
    ny: isize,
) -> (f64, f64) {
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    let mut count = 0.0;
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            if xx >= 0 && xx < nx && yy >= 0 && yy < ny {
                let v = vals[slice_offset + xx as usize + yy as usize * nx as usize];
                sum += v;
                sum2 += v * v;
                count += 1.0;
            }
        }
    }
    if count < 1.0 {
        return (0.0, f64::INFINITY);
    }
    let mean = sum / count;
    let var = sum2 / count - mean * mean;
    (mean, var.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_kuwahara() {
        let img = ImageData::from_function(
            [12, 12, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| {
                if x > 6.0 {
                    100.0
                } else {
                    0.0
                }
            },
        );
        let r = kuwahara_filter(&img, "v", 2);
        assert_eq!(r.dimensions(), [12, 12, 1]);
    }

    #[test]
    fn filters_each_z_slice_independently() {
        let mut img = ImageData::with_dimensions(3, 3, 2);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![
                    1.0, 1.0, 1.0, 1.0, 9.0, 1.0, 1.0, 1.0, 1.0, 10.0, 10.0, 10.0, 10.0, 90.0,
                    10.0, 10.0, 10.0, 10.0,
                ],
                1,
            )));

        let r = kuwahara_filter(&img, "v", 1);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(13, &mut buf);
        assert!(buf[0] >= 10.0);
    }
}
