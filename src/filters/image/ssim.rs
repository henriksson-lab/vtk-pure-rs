use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Structural Similarity Index (SSIM) between two ImageData objects.
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

    if a.extent() != b.extent() || aa.num_components() != ba.num_components() {
        return (a.clone(), 0.0);
    }

    let dims = a.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let num_components = aa.num_components();
    if n == 0 || num_components == 0 || aa.num_tuples() < n || ba.num_tuples() < n {
        return (a.clone(), 0.0);
    }
    let patch_radius = radius;
    let r = patch_radius as i64;

    let mut buf_a = vec![0.0f64; num_components];
    let mut buf_b = vec![0.0f64; num_components];
    let mut va = vec![0.0f64; n * num_components];
    let mut vb = vec![0.0f64; n * num_components];
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        for c in 0..num_components {
            va[i * num_components + c] = buf_a[c];
            vb[i * num_components + c] = buf_b[c];
        }
    }

    let mut constants = vec![[0.0f64; 2]; num_components];
    for c in 0..num_components {
        let mut min_a = f64::INFINITY;
        let mut max_a = f64::NEG_INFINITY;
        let mut min_b = f64::INFINITY;
        let mut max_b = f64::NEG_INFINITY;

        for i in 0..n {
            let av = va[i * num_components + c];
            let bv = vb[i * num_components + c];
            min_a = min_a.min(av);
            max_a = max_a.max(av);
            min_b = min_b.min(bv);
            max_b = max_b.max(bv);
        }

        let range = (max_a - min_a).max(max_b - min_b);
        constants[c][0] = 0.0001 * range * range;
        constants[c][1] = 0.0009 * range * range;
    }

    let index = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };
    let get_a =
        |i: usize, j: usize, k: usize, c: usize| -> f64 { va[index(i, j, k) * num_components + c] };
    let get_b =
        |i: usize, j: usize, k: usize, c: usize| -> f64 { vb[index(i, j, k) * num_components + c] };

    let mut ssim_map = vec![0.0f64; n * num_components];
    let squared_radius = r * r;
    let has_x_thickness = nx > 1;
    let has_y_thickness = ny > 1;
    let has_z_thickness = nz > 1;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let imin = if has_x_thickness {
                    i.saturating_sub(patch_radius)
                } else {
                    0
                };
                let imax = if has_x_thickness {
                    i.saturating_add(patch_radius).min(nx - 1)
                } else {
                    0
                };
                let jmin = if has_y_thickness {
                    j.saturating_sub(patch_radius)
                } else {
                    0
                };
                let jmax = if has_y_thickness {
                    j.saturating_add(patch_radius).min(ny - 1)
                } else {
                    0
                };
                let kmin = if has_z_thickness {
                    k.saturating_sub(patch_radius)
                } else {
                    0
                };
                let kmax = if has_z_thickness {
                    k.saturating_add(patch_radius).min(nz - 1)
                } else {
                    0
                };

                for c in 0..num_components {
                    let mut mean_a = 0.0;
                    let mut mean_b = 0.0;
                    let mut total_weights = 0.0;

                    for dz in kmin..=kmax {
                        for dy in jmin..=jmax {
                            for dx in imin..=imax {
                                let ddx = i as i64 - dx as i64;
                                let ddy = j as i64 - dy as i64;
                                let ddz = k as i64 - dz as i64;
                                if ddx * ddx + ddy * ddy + ddz * ddz <= squared_radius {
                                    mean_a += get_a(dx, dy, dz, c);
                                    mean_b += get_b(dx, dy, dz, c);
                                    total_weights += 1.0;
                                }
                            }
                        }
                    }

                    mean_a /= total_weights;
                    mean_b /= total_weights;

                    let mut var_a = 0.0;
                    let mut var_b = 0.0;
                    let mut cov_ab = 0.0;
                    for dz in kmin..=kmax {
                        for dy in jmin..=jmax {
                            for dx in imin..=imax {
                                let ddx = i as i64 - dx as i64;
                                let ddy = j as i64 - dy as i64;
                                let ddz = k as i64 - dz as i64;
                                if ddx * ddx + ddy * ddy + ddz * ddz <= squared_radius {
                                    let av = get_a(dx, dy, dz, c);
                                    let bv = get_b(dx, dy, dz, c);
                                    var_a += (av - mean_a) * (av - mean_a);
                                    var_b += (bv - mean_b) * (bv - mean_b);
                                    cov_ab += (av - mean_a) * (bv - mean_b);
                                }
                            }
                        }
                    }
                    var_a /= total_weights;
                    var_b /= total_weights;
                    cov_ab /= total_weights;

                    let c1 = constants[c][0];
                    let c2 = constants[c][1];
                    ssim_map[index(i, j, k) * num_components + c] = (2.0 * (mean_a * mean_b) + c1)
                        * (2.0 * cov_ab + c2)
                        / ((mean_a * mean_a + mean_b * mean_b + c1) * (var_a + var_b + c2));
                }
            }
        }
    }

    let mean_ssim = ssim_map.iter().sum::<f64>() / (n * num_components) as f64;

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SSIM",
            ssim_map,
            num_components,
        )));
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
