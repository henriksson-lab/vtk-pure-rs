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
    if aa.num_components() != ba.num_components() {
        return (a.clone(), 0.0);
    }

    let dims = a.dimensions();
    if dims != b.dimensions() {
        return (a.clone(), 0.0);
    }
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;
    let nc = aa.num_components();
    if n == 0 || nz == 0 {
        return (a.clone(), 0.0);
    }
    if aa.num_tuples() < n || ba.num_tuples() < n {
        return (a.clone(), 0.0);
    }
    let r = radius as i64;

    let mut buf_a = vec![0.0f64; nc];
    let mut buf_b = vec![0.0f64; nc];
    let mut va = vec![0.0f64; n * nc];
    let mut vb = vec![0.0f64; n * nc];
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        let out = i * nc;
        va[out..out + nc].copy_from_slice(&buf_a);
        vb[out..out + nc].copy_from_slice(&buf_b);
    }

    let c1 = 0.01 * 0.01 * 255.0 * 255.0;
    let c2 = 0.03 * 0.03 * 255.0 * 255.0;

    let mut ssim_map = vec![0.0f64; n * nc];
    let xy = nx * ny;
    let full_3d = nz > 1;

    for comp in 0..nc {
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let mut ma = 0.0;
                    let mut mb = 0.0;
                    let mut sa2 = 0.0;
                    let mut sb2 = 0.0;
                    let mut sab = 0.0;
                    let mut count = 0.0;
                    let squared_radius = r * r;
                    let z_min = if full_3d { k as i64 - r } else { 0 };
                    let z_max = if full_3d { k as i64 + r } else { 0 };

                    for z in z_min..=z_max {
                        for y in (j as i64 - r)..=(j as i64 + r) {
                            for x in (i as i64 - r)..=(i as i64 + r) {
                                let dx = x - i as i64;
                                let dy = y - j as i64;
                                let dz = if full_3d { z - k as i64 } else { 0 };
                                if x < 0
                                    || y < 0
                                    || z < 0
                                    || x >= nx as i64
                                    || y >= ny as i64
                                    || z >= nz as i64
                                    || dx * dx + dy * dy + dz * dz > squared_radius
                                {
                                    continue;
                                }
                                let idx =
                                    (z as usize * xy + y as usize * nx + x as usize) * nc + comp;
                                let a_v = va[idx];
                                let b_v = vb[idx];
                                ma += a_v;
                                mb += b_v;
                                sa2 += a_v * a_v;
                                sb2 += b_v * b_v;
                                sab += a_v * b_v;
                                count += 1.0;
                            }
                        }
                    }
                    if count == 0.0 {
                        continue;
                    }
                    ma /= count;
                    mb /= count;
                    let var_a = sa2 / count - ma * ma;
                    let var_b = sb2 / count - mb * mb;
                    let cov_ab = sab / count - ma * mb;

                    ssim_map[(k * xy + j * nx + i) * nc + comp] = (2.0 * ma * mb + c1)
                        * (2.0 * cov_ab + c2)
                        / ((ma * ma + mb * mb + c1) * (var_a + var_b + c2));
                }
            }
        }
    }

    let mean_ssim = ssim_map.iter().sum::<f64>() / ssim_map.len() as f64;

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("SSIM", ssim_map, nc)));
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
