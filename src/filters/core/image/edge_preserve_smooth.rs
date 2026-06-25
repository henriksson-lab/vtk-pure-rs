use crate::data::{AnyDataArray, DataArray, ImageData};

/// Non-local means denoising on ImageData.
///
/// For each voxel, computes a weighted average using patch similarity.
/// Patches with similar neighborhoods contribute more. Adds "NLMeans" array.
pub fn image_non_local_means(
    input: &ImageData,
    scalars: &str,
    patch_radius: usize,
    search_radius: usize,
    h: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;
    let pr = patch_radius.max(1) as i64;
    let sr = search_radius.max(1) as i64;
    let h2 = h * h;

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

    let mut result = vec![0.0f64; n];

    for j in 0..ny {
        for i in 0..nx {
            let mut sum_w = 0.0;
            let mut sum_v = 0.0;
            let ii = i as i64;
            let jj = j as i64;

            for sj in -sr..=sr {
                for si in -sr..=sr {
                    let ni = ii + si;
                    let nj = jj + sj;
                    // Compute patch distance
                    let mut patch_d2 = 0.0;
                    for pj in -pr..=pr {
                        for pi in -pr..=pr {
                            let d = get(ii + pi, jj + pj) - get(ni + pi, nj + pj);
                            patch_d2 += d * d;
                        }
                    }
                    let w = (-patch_d2 / h2).exp();
                    sum_w += w;
                    sum_v += w * get(ni, nj);
                }
            }

            result[j * nx + i] = if sum_w > 1e-15 {
                sum_v / sum_w
            } else {
                values[j * nx + i]
            };
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("NLMeans", result, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denoises_spike() {
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values = vec![50.0; 49];
        values[24] = 200.0; // noisy spike
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_non_local_means(&img, "v", 1, 2, 500.0); // large h -> strong smoothing
        let arr = result.point_data().get_array("NLMeans").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(24, &mut buf);
        assert!(buf[0] < 200.0, "val={}", buf[0]); // spike reduced
    }

    #[test]
    fn preserves_uniform() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0; 25],
                1,
            )));

        let result = image_non_local_means(&img, "v", 1, 1, 5.0);
        let arr = result.point_data().get_array("NLMeans").unwrap();
        let mut buf = [0.0f64];
        for i in 0..25 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 10.0).abs() < 1e-5);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_non_local_means(&img, "nope", 1, 1, 5.0);
        assert!(r.point_data().get_array("NLMeans").is_none());
    }
}
