use crate::data::{AnyDataArray, DataArray, ImageData};

/// Estimate translation offset between two 2D images via cross-correlation.
///
/// Computes normalized cross-correlation for a range of offsets and
/// returns the (dx, dy) with highest correlation. Brute force for small images.
pub fn register_translation_2d(
    fixed: &ImageData,
    moving_img: &ImageData,
    scalars: &str,
    max_shift: usize,
) -> (i64, i64, f64) {
    let fa = match fixed.point_data().get_array(scalars) {
        Some(a) => a,
        None => return (0, 0, 0.0),
    };
    let ma = match moving_img.point_data().get_array(scalars) {
        Some(a) => a,
        None => return (0, 0, 0.0),
    };

    let fd = fixed.dimensions();
    let nx = fd[0] as usize;
    let ny = fd[1] as usize;
    let md = moving_img.dimensions();
    let mnx = md[0] as usize;
    let mny = md[1] as usize;

    let mut fb = [0.0f64];
    let mut mb = [0.0f64];
    let fv: Vec<f64> = (0..nx * ny)
        .map(|i| {
            fa.tuple_as_f64(i, &mut fb);
            fb[0]
        })
        .collect();
    let mv: Vec<f64> = (0..mnx * mny)
        .map(|i| {
            ma.tuple_as_f64(i, &mut mb);
            mb[0]
        })
        .collect();

    let s = max_shift as i64;
    let mut best_corr = f64::MIN;
    let mut best_dx = 0i64;
    let mut best_dy = 0i64;

    for dy in -s..=s {
        for dx in -s..=s {
            let mut sum_fm = 0.0;
            let mut sum_f2 = 0.0;
            let mut sum_m2 = 0.0;
            let mut count = 0;
            for j in 0..ny {
                let mj = j as i64 + dy;
                if mj < 0 || mj >= mny as i64 {
                    continue;
                }
                for i in 0..nx {
                    let mi = i as i64 + dx;
                    if mi < 0 || mi >= mnx as i64 {
                        continue;
                    }
                    let f = fv[j * nx + i];
                    let m = mv[mj as usize * mnx + mi as usize];
                    sum_fm += f * m;
                    sum_f2 += f * f;
                    sum_m2 += m * m;
                    count += 1;
                }
            }
            if count > 0 && sum_f2 > 1e-15 && sum_m2 > 1e-15 {
                let ncc = sum_fm / (sum_f2.sqrt() * sum_m2.sqrt());
                if ncc > best_corr {
                    best_corr = ncc;
                    best_dx = dx;
                    best_dy = dy;
                }
            }
        }
    }

    (best_dx, best_dy, best_corr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = (0..25).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let (dx, dy, corr) = register_translation_2d(&img, &img, "v", 2);
        assert_eq!(dx, 0);
        assert_eq!(dy, 0);
        assert!(corr > 0.99);
    }

    #[test]
    fn shifted_image() {
        let mut fixed = ImageData::with_dimensions(8, 8, 1);
        let mut moving = ImageData::with_dimensions(8, 8, 1);
        let mut fv = vec![0.0; 64];
        let mut mv = vec![0.0; 64];
        // Bright spot at (3,3) in fixed, (4,3) in moving -> dx=1
        fv[3 * 8 + 3] = 100.0;
        fv[3 * 8 + 4] = 80.0;
        mv[3 * 8 + 4] = 100.0;
        mv[3 * 8 + 5] = 80.0;
        fixed
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", fv, 1)));
        moving
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", mv, 1)));

        let (dx, _, _) = register_translation_2d(&fixed, &moving, "v", 3);
        assert_eq!(dx, 1);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let (dx, dy, _) = register_translation_2d(&img, &img, "nope", 2);
        assert_eq!(dx, 0);
        assert_eq!(dy, 0);
    }
}
