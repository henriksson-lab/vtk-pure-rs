//! Local histogram features (entropy, mode, range) per pixel neighborhood.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local entropy in a sliding window.
pub fn local_entropy(input: &ImageData, scalars: &str, radius: usize, bins: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let slice = nx.saturating_mul(ny);
    let n = slice.saturating_mul(nz);
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
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let r = radius as isize;
    let bins = bins.max(2);

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let slice_offset = idx / slice * slice;
            let in_slice = idx - slice_offset;
            let iy = in_slice / nx;
            let ix = in_slice % nx;
            let mut hist = vec![0usize; bins];
            let mut count = 0usize;
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                        let v = vals[slice_offset + sx as usize + sy as usize * nx];
                        let bi = (((v - mn) / range * bins as f64).floor() as usize).min(bins - 1);
                        hist[bi] += 1;
                        count += 1;
                    }
                }
            }
            if count == 0 {
                return 0.0;
            }
            let cf = count as f64;
            hist.iter()
                .filter(|&&h| h > 0)
                .map(|&h| {
                    let p = h as f64 / cf;
                    -p * p.ln()
                })
                .sum::<f64>()
        })
        .collect();

    ImageData::with_dimensions(nx, ny, nz)
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Entropy", data, 1)))
}

/// Compute local mode (most frequent bin value) in a sliding window.
pub fn local_mode(input: &ImageData, scalars: &str, radius: usize, bins: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let slice = nx.saturating_mul(ny);
    let n = slice.saturating_mul(nz);
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
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
    let r = radius as isize;
    let bins = bins.max(2);

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let slice_offset = idx / slice * slice;
            let in_slice = idx - slice_offset;
            let iy = in_slice / nx;
            let ix = in_slice % nx;
            let mut hist = vec![0usize; bins];
            for dy in -r..=r {
                for dx in -r..=r {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx >= 0 && sx < nx as isize && sy >= 0 && sy < ny as isize {
                        let v = vals[slice_offset + sx as usize + sy as usize * nx];
                        let bi = (((v - mn) / range * bins as f64).floor() as usize).min(bins - 1);
                        hist[bi] += 1;
                    }
                }
            }
            let best = hist
                .iter()
                .enumerate()
                .max_by_key(|(_, &c)| c)
                .map(|(i, _)| i)
                .unwrap_or(0);
            mn + (best as f64 + 0.5) / bins as f64 * range
        })
        .collect();

    ImageData::with_dimensions(nx, ny, nz)
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Mode", data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_entropy() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = local_entropy(&img, "v", 2, 8);
        assert!(r.point_data().get_array("Entropy").is_some());
    }
    #[test]
    fn test_mode() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 5.0,
        );
        let r = local_mode(&img, "v", 1, 10);
        assert!(r.point_data().get_array("Mode").is_some());
    }

    #[test]
    fn entropy_processes_z_slices_independently() {
        let mut img = ImageData::with_dimensions(2, 2, 2);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 2.0],
                1,
            )));

        let r = local_entropy(&img, "v", 1, 2);
        let arr = r.point_data().get_array("Entropy").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn mode_processes_z_slices_independently() {
        let mut img = ImageData::with_dimensions(2, 2, 2);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 1.0, 1.0, 1.0, 10.0, 10.0, 10.0, 10.0],
                1,
            )));

        let r = local_mode(&img, "v", 1, 4);
        let arr = r.point_data().get_array("Mode").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 5.0);
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 5.0);
    }
}
