use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Structural Similarity Index (SSIM) between two 2D ImageData.
///
/// Returns a per-pixel SSIM map and the mean SSIM value.
/// SSIM combines luminance, contrast, and structure comparison.
pub fn image_ssim(a: &ImageData, b: &ImageData, scalars: &str, radius: usize) -> (ImageData, f64) {
    let aa = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return (a.clone(), 0.0),
    };
    let ba = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return (a.clone(), 0.0),
    };

    let dims = a.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;
    let r = radius.max(1) as i64;

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];
    let va: Vec<f64> = (0..n)
        .map(|i| {
            aa.tuple_as_f64(i, &mut buf_a);
            buf_a[0]
        })
        .collect();
    let vb: Vec<f64> = (0..n)
        .map(|i| {
            ba.tuple_as_f64(i, &mut buf_b);
            buf_b[0]
        })
        .collect();

    let c1 = 0.01 * 0.01 * 255.0 * 255.0;
    let c2 = 0.03 * 0.03 * 255.0 * 255.0;

    let get_a = |i: i64, j: i64| -> f64 {
        va[(j.clamp(0, ny as i64 - 1) as usize) * nx + (i.clamp(0, nx as i64 - 1) as usize)]
    };
    let get_b = |i: i64, j: i64| -> f64 {
        vb[(j.clamp(0, ny as i64 - 1) as usize) * nx + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    let mut ssim_map = vec![0.0f64; n];

    for j in 0..ny {
        for i in 0..nx {
            let mut ma = 0.0;
            let mut mb = 0.0;
            let mut sa2 = 0.0;
            let mut sb2 = 0.0;
            let mut sab = 0.0;
            let mut cnt = 0.0;
            for dj in -r..=r {
                for di in -r..=r {
                    let a_v = get_a(i as i64 + di, j as i64 + dj);
                    let b_v = get_b(i as i64 + di, j as i64 + dj);
                    ma += a_v;
                    mb += b_v;
                    sa2 += a_v * a_v;
                    sb2 += b_v * b_v;
                    sab += a_v * b_v;
                    cnt += 1.0;
                }
            }
            ma /= cnt;
            mb /= cnt;
            let var_a = sa2 / cnt - ma * ma;
            let var_b = sb2 / cnt - mb * mb;
            let cov_ab = sab / cnt - ma * mb;

            ssim_map[j * nx + i] = (2.0 * ma * mb + c1) * (2.0 * cov_ab + c2)
                / ((ma * ma + mb * mb + c1) * (var_a + var_b + c2));
        }
    }

    let mean_ssim = ssim_map.iter().sum::<f64>() / n as f64;

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("SSIM", ssim_map, 1)));
    (img, mean_ssim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_ssim_1() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64 * 10.0).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let (_, mean) = image_ssim(&img, &img, "v", 1);
        assert!(mean > 0.99);
    }

    #[test]
    fn different_images_lower() {
        let mut a = ImageData::with_dimensions(5, 5, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0; 25],
                1,
            )));
        let mut b = ImageData::with_dimensions(5, 5, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![255.0; 25],
                1,
            )));

        let (_, mean) = image_ssim(&a, &b, "v", 1);
        assert!(mean < 0.5);
    }

    #[test]
    fn has_ssim_array() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let (result, _) = image_ssim(&img, &img, "v", 1);
        assert!(result.point_data().get_array("SSIM").is_some());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let (_, mean) = image_ssim(&img, &img, "nope", 1);
        assert_eq!(mean, 0.0);
    }
}
