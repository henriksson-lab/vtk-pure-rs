use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute local entropy of an ImageData scalar field.
///
/// For each voxel, computes Shannon entropy of values in a neighborhood
/// of given radius. High entropy = high local variability.
/// Adds "Entropy" scalar array.
pub fn image_entropy(input: &ImageData, scalars: &str, radius: usize, n_bins: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let r = radius.max(1) as i64;
    let nb = n_bins.max(2);

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    // Global min/max for binning
    let mut gmin = f64::MAX;
    let mut gmax = f64::MIN;
    for &v in &values {
        gmin = gmin.min(v);
        gmax = gmax.max(v);
    }
    let range = (gmax - gmin).max(1e-15);

    let mut entropy = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut hist = vec![0usize; nb];
                let mut count = 0;

                for dk in -r..=r {
                    for dj in -r..=r {
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                            let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            let bin = ((v - gmin) / range * (nb as f64 - 1.0)).round() as usize;
                            hist[bin.min(nb - 1)] += 1;
                            count += 1;
                        }
                    }
                }

                let mut e = 0.0;
                let cf = count as f64;
                for &h in &hist {
                    if h > 0 {
                        let p = h as f64 / cf;
                        e -= p * p.ln();
                    }
                }
                entropy[k * ny * nx + j * nx + i] = e;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Entropy", entropy, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_low_entropy() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0; 25],
                1,
            )));

        let result = image_entropy(&img, "v", 1, 10);
        let arr = result.point_data().get_array("Entropy").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] < 0.01); // all same value -> zero entropy
    }

    #[test]
    fn varied_high_entropy() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_entropy(&img, "v", 2, 10);
        let arr = result.point_data().get_array("Entropy").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!(buf[0] > 0.5); // varied values -> high entropy
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_entropy(&img, "nope", 1, 10);
        assert!(r.point_data().get_array("Entropy").is_none());
    }
}
