//! Non-local means denoising.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Non-local means denoising. Compares patches to compute weights.
pub fn non_local_means(
    input: &ImageData,
    scalars: &str,
    search_radius: usize,
    patch_radius: usize,
    h: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let h2 = h * h;
    let sr = search_radius as isize;
    let pr = patch_radius as isize;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            let mut wsum = 0.0;
            let mut vsum = 0.0;
            for dy in -sr..=sr {
                for dx in -sr..=sr {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx < 0 || sx >= nx as isize || sy < 0 || sy >= ny as isize {
                        continue;
                    }
                    let d2 = patch_distance(&vals, ix, iy, sx as usize, sy as usize, nx, ny, pr);
                    let w = (-d2 / h2).exp();
                    wsum += w;
                    vsum += w * vals[sx as usize + sy as usize * nx];
                }
            }
            if wsum > 1e-15 {
                vsum / wsum
            } else {
                vals[idx]
            }
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

fn patch_distance(
    vals: &[f64],
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    nx: usize,
    ny: usize,
    pr: isize,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0.0;
    for dy in -pr..=pr {
        for dx in -pr..=pr {
            let a = safe_get(vals, x1 as isize + dx, y1 as isize + dy, nx, ny);
            let b = safe_get(vals, x2 as isize + dx, y2 as isize + dy, nx, ny);
            sum += (a - b) * (a - b);
            count += 1.0;
        }
    }
    sum / (count as f64).max(1.0)
}

fn safe_get(vals: &[f64], x: isize, y: isize, nx: usize, ny: usize) -> f64 {
    let cx = x.clamp(0, nx as isize - 1) as usize;
    let cy = y.clamp(0, ny as isize - 1) as usize;
    vals[cx + cy * nx]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nlm() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x * 10.0,
        );
        let r = non_local_means(&img, "v", 2, 1, 10.0);
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
}
