use crate::data::{AnyDataArray, DataArray, ImageData};

/// Chamfer distance transform on binary ImageData.
///
/// Uses a two-pass (forward/backward) approach with 3-4-5 integer weights
/// that approximate Euclidean distance. The 3-4-5 weights correspond to
/// face-adjacent (3), edge-adjacent (4), and corner-adjacent (5) neighbors
/// in a 3x3x3 neighborhood.
///
/// The input array is treated as binary: voxels with value >= 0.5 are
/// foreground (distance 0), all others are background.
///
/// The result is scaled by 1/3 so that face-adjacent distance is ~1.0.
pub fn chamfer_distance_transform(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    let big: i64 = 1_000_000;

    // Initialize: 0 for foreground, big for background
    let mut dist: Vec<i64> = vec![big; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= 0.5 {
            dist[i] = 0;
        }
    }

    let idx = |x: usize, y: usize, z: usize| -> usize { z * ny * nx + y * nx + x };

    // Forward pass: scan in +x, +y, +z order
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let cur: usize = idx(x, y, z);
                if dist[cur] == 0 {
                    continue;
                }

                // Face neighbors (weight 3)
                if x > 0 {
                    let v: i64 = dist[idx(x - 1, y, z)] + 3;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if y > 0 {
                    let v: i64 = dist[idx(x, y - 1, z)] + 3;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if z > 0 {
                    let v: i64 = dist[idx(x, y, z - 1)] + 3;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }

                // Edge-diagonal neighbors (weight 4)
                if x > 0 && y > 0 {
                    let v: i64 = dist[idx(x - 1, y - 1, z)] + 4;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if x > 0 && z > 0 {
                    let v: i64 = dist[idx(x - 1, y, z - 1)] + 4;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if y > 0 && z > 0 {
                    let v: i64 = dist[idx(x, y - 1, z - 1)] + 4;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }

                // Corner-diagonal neighbors (weight 5)
                if x > 0 && y > 0 && z > 0 {
                    let v: i64 = dist[idx(x - 1, y - 1, z - 1)] + 5;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
            }
        }
    }

    // Backward pass: scan in -x, -y, -z order
    for z in (0..nz).rev() {
        for y in (0..ny).rev() {
            for x in (0..nx).rev() {
                let cur: usize = idx(x, y, z);
                if dist[cur] == 0 {
                    continue;
                }

                // Face neighbors (weight 3)
                if x + 1 < nx {
                    let v: i64 = dist[idx(x + 1, y, z)] + 3;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if y + 1 < ny {
                    let v: i64 = dist[idx(x, y + 1, z)] + 3;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if z + 1 < nz {
                    let v: i64 = dist[idx(x, y, z + 1)] + 3;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }

                // Edge-diagonal neighbors (weight 4)
                if x + 1 < nx && y + 1 < ny {
                    let v: i64 = dist[idx(x + 1, y + 1, z)] + 4;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if x + 1 < nx && z + 1 < nz {
                    let v: i64 = dist[idx(x + 1, y, z + 1)] + 4;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
                if y + 1 < ny && z + 1 < nz {
                    let v: i64 = dist[idx(x, y + 1, z + 1)] + 4;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }

                // Corner-diagonal neighbors (weight 5)
                if x + 1 < nx && y + 1 < ny && z + 1 < nz {
                    let v: i64 = dist[idx(x + 1, y + 1, z + 1)] + 5;
                    if v < dist[cur] {
                        dist[cur] = v;
                    }
                }
            }
        }
    }

    // Convert to f64, scale by 1/3 so face-adjacent distance ~ 1.0
    let scale: f64 = 1.0 / 3.0;
    let result: Vec<f64> = dist.iter().map(|&d| d as f64 * scale).collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ChamferDistance",
            result,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_seed_1d() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let mut values: Vec<f64> = vec![0.0; 7];
        values[3] = 1.0; // seed at center
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));

        let result = chamfer_distance_transform(&img, "mask");
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];

        // Seed voxel should be 0
        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);

        // Neighbors at distance 1 should be 3/3 = 1.0
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);

        // Distance should increase monotonically from center
        arr.tuple_as_f64(0, &mut buf);
        let d0: f64 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let d1: f64 = buf[0];
        assert!(d0 > d1);
    }

    #[test]
    fn all_foreground_zero_distance() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let values: Vec<f64> = vec![1.0; 9];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));

        let result = chamfer_distance_transform(&img, "mask");
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn missing_array_returns_copy() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = chamfer_distance_transform(&img, "nonexistent");
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }
}
