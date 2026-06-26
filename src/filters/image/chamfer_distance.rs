use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute chamfer distance transform using 3-4-5 weights.
///
/// More accurate than the simple forward-backward pass. Uses the
/// 3-4-5 Borgefors chamfer mask for better Euclidean approximation.
/// Adds "ChamferDistance" array.
pub fn image_chamfer_distance(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let big = 1e8f64;
    let idx = |x: usize, y: usize, z: usize| -> usize { z * nx * ny + y * nx + x };

    let mut buf = [0.0f64];
    let mut dist: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                0.0
            } else {
                big
            }
        })
        .collect();

    // Forward pass (top-left-front to bottom-right-back)
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let cur = idx(x, y, z);
                for dz in -1isize..=0 {
                    for dy in -1isize..=1 {
                        for dx in -1isize..=1 {
                            if dz == 0 && (dy > 0 || (dy == 0 && dx >= 0)) {
                                continue;
                            }
                            let xx = x as isize + dx;
                            let yy = y as isize + dy;
                            let zz = z as isize + dz;
                            if xx < 0
                                || yy < 0
                                || zz < 0
                                || xx >= nx as isize
                                || yy >= ny as isize
                                || zz >= nz as isize
                            {
                                continue;
                            }
                            let changed = (dx != 0) as u8 + (dy != 0) as u8 + (dz != 0) as u8;
                            let weight = match changed {
                                1 => 3.0,
                                2 => 4.0,
                                3 => 5.0,
                                _ => continue,
                            };
                            dist[cur] = dist[cur]
                                .min(dist[idx(xx as usize, yy as usize, zz as usize)] + weight);
                        }
                    }
                }
            }
        }
    }

    // Backward pass (bottom-right-back to top-left-front)
    for z in (0..nz).rev() {
        for y in (0..ny).rev() {
            for x in (0..nx).rev() {
                let cur = idx(x, y, z);
                for dz in 0isize..=1 {
                    for dy in -1isize..=1 {
                        for dx in -1isize..=1 {
                            if dz == 0 && (dy < 0 || (dy == 0 && dx <= 0)) {
                                continue;
                            }
                            let xx = x as isize + dx;
                            let yy = y as isize + dy;
                            let zz = z as isize + dz;
                            if xx < 0
                                || yy < 0
                                || zz < 0
                                || xx >= nx as isize
                                || yy >= ny as isize
                                || zz >= nz as isize
                            {
                                continue;
                            }
                            let changed = (dx != 0) as u8 + (dy != 0) as u8 + (dz != 0) as u8;
                            let weight = match changed {
                                1 => 3.0,
                                2 => 4.0,
                                3 => 5.0,
                                _ => continue,
                            };
                            dist[cur] = dist[cur]
                                .min(dist[idx(xx as usize, yy as usize, zz as usize)] + weight);
                        }
                    }
                }
            }
        }
    }

    // Scale to approximate Euclidean (divide by 3)
    for d in &mut dist {
        *d /= 3.0;
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ChamferDistance",
            dist,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_seed() {
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values = vec![0.0; 49];
        values[24] = 1.0; // center seed
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_chamfer_distance(&img, "v", 0.5);
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(24, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(25, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.5);
    }

    #[test]
    fn single_seed_3d_corner_weight() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let mut values = vec![0.0; 27];
        values[13] = 1.0;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_chamfer_distance(&img, "v", 0.5);
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn all_foreground() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = image_chamfer_distance(&img, "v", 0.5);
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_chamfer_distance(&img, "nope", 0.5);
        assert!(r.point_data().get_array("ChamferDistance").is_none());
    }
}
