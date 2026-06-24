//! Euclidean distance field computation for binary ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute approximate Euclidean distance transform of a binary image.
///
/// For each voxel, computes the distance to the nearest foreground voxel.
pub fn distance_transform(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let spacing = image.spacing();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];
    let mut is_fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();

    // Multi-pass Chamfer distance transform (3-4-5 weights)
    let mut dist = vec![f64::MAX; n];
    for i in 0..n {
        if is_fg[i] {
            dist[i] = 0.0;
        }
    }

    // Forward pass
    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if dist[idx] == 0.0 {
                    continue;
                }
                let neighbors: Vec<(i64, i64, i64, f64)> = vec![
                    (-1, 0, 0, spacing[0]),
                    (0, -1, 0, spacing[1]),
                    (0, 0, -1, spacing[2]),
                    (
                        -1,
                        -1,
                        0,
                        (spacing[0] * spacing[0] + spacing[1] * spacing[1]).sqrt(),
                    ),
                    (
                        1,
                        -1,
                        0,
                        (spacing[0] * spacing[0] + spacing[1] * spacing[1]).sqrt(),
                    ),
                    (
                        -1,
                        0,
                        -1,
                        (spacing[0] * spacing[0] + spacing[2] * spacing[2]).sqrt(),
                    ),
                    (
                        0,
                        -1,
                        -1,
                        (spacing[1] * spacing[1] + spacing[2] * spacing[2]).sqrt(),
                    ),
                ];
                for (dx, dy, dz, w) in &neighbors {
                    let nx = ix as i64 + dx;
                    let ny = iy as i64 + dy;
                    let nz = iz as i64 + dz;
                    if nx >= 0
                        && ny >= 0
                        && nz >= 0
                        && (nx as usize) < dims[0]
                        && (ny as usize) < dims[1]
                        && (nz as usize) < dims[2]
                    {
                        let ni =
                            nx as usize + ny as usize * dims[0] + nz as usize * dims[0] * dims[1];
                        let nd = dist[ni] + w;
                        if nd < dist[idx] {
                            dist[idx] = nd;
                        }
                    }
                }
            }
        }
    }

    // Backward pass
    for iz in (0..dims[2]).rev() {
        for iy in (0..dims[1]).rev() {
            for ix in (0..dims[0]).rev() {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                if dist[idx] == 0.0 {
                    continue;
                }
                let neighbors: Vec<(i64, i64, i64, f64)> = vec![
                    (1, 0, 0, spacing[0]),
                    (0, 1, 0, spacing[1]),
                    (0, 0, 1, spacing[2]),
                    (
                        1,
                        1,
                        0,
                        (spacing[0] * spacing[0] + spacing[1] * spacing[1]).sqrt(),
                    ),
                    (
                        -1,
                        1,
                        0,
                        (spacing[0] * spacing[0] + spacing[1] * spacing[1]).sqrt(),
                    ),
                    (
                        1,
                        0,
                        1,
                        (spacing[0] * spacing[0] + spacing[2] * spacing[2]).sqrt(),
                    ),
                    (
                        0,
                        1,
                        1,
                        (spacing[1] * spacing[1] + spacing[2] * spacing[2]).sqrt(),
                    ),
                ];
                for (dx, dy, dz, w) in &neighbors {
                    let nx = ix as i64 + dx;
                    let ny = iy as i64 + dy;
                    let nz = iz as i64 + dz;
                    if nx >= 0
                        && ny >= 0
                        && nz >= 0
                        && (nx as usize) < dims[0]
                        && (ny as usize) < dims[1]
                        && (nz as usize) < dims[2]
                    {
                        let ni =
                            nx as usize + ny as usize * dims[0] + nz as usize * dims[0] * dims[1];
                        let nd = dist[ni] + w;
                        if nd < dist[idx] {
                            dist[idx] = nd;
                        }
                    }
                }
            }
        }
    }

    for d in &mut dist {
        if *d == f64::MAX {
            *d = -1.0;
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DistanceField",
            dist,
            1,
        )));
    result
}

/// Compute signed distance field: negative inside foreground, positive outside.
pub fn signed_distance_transform(image: &ImageData, array_name: &str) -> ImageData {
    let outside = distance_transform(image, array_name);
    // Invert and compute inside distance
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return outside,
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];

    // Create inverted mask
    let mut inv_vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                0.0
            } else {
                1.0
            }
        })
        .collect();

    let mut inv_image = image.clone();
    inv_image
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("_inv", inv_vals, 1)));
    let inside = distance_transform(&inv_image, "_inv");

    let out_arr = outside.point_data().get_array("DistanceField").unwrap();
    let in_arr = inside.point_data().get_array("DistanceField").unwrap();

    let mut sdf = Vec::with_capacity(n);
    let mut ob = [0.0f64];
    let mut ib = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        out_arr.tuple_as_f64(i, &mut ob);
        in_arr.tuple_as_f64(i, &mut ib);
        sdf.push(if buf[0] > 0.5 {
            -ib[0].max(0.0)
        } else {
            ob[0].max(0.0)
        });
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SignedDistance",
            sdf,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distance() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "mask",
            |x, y, _| {
                if (x - 10.0).powi(2) + (y - 10.0).powi(2) < 25.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = distance_transform(&img, "mask");
        let arr = result.point_data().get_array("DistanceField").unwrap();
        let mut buf = [0.0f64];
        // Center should be 0 (inside)
        arr.tuple_as_f64(10 + 10 * 20, &mut buf);
        assert_eq!(buf[0], 0.0);
        // Far corner should have positive distance
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0);
    }
    #[test]
    fn signed() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "mask",
            |x, y, _| {
                if (x - 5.0).powi(2) + (y - 5.0).powi(2) < 9.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = signed_distance_transform(&img, "mask");
        let arr = result.point_data().get_array("SignedDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(5 + 5 * 10, &mut buf);
        assert!(buf[0] < 0.0); // inside = negative
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0); // outside = positive
    }
}
