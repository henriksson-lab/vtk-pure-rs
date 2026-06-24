//! 2D/3D image feature detection: blobs, edges, corners.

use crate::data::{AnyDataArray, DataArray, ImageData, Points, PolyData};

/// Detect blobs (local maxima) in a 3D scalar field.
///
/// Returns a PolyData with point locations of detected blobs.
pub fn detect_blobs_3d(image: &ImageData, array_name: &str, min_value: f64) -> PolyData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();
    let total = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..total)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();

    let val_at = |ix: usize, iy: usize, iz: usize| -> f64 {
        let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
        if idx < vals.len() {
            vals[idx]
        } else {
            0.0
        }
    };

    let mut pts = Points::<f64>::new();
    let mut strengths = Vec::new();

    for iz in 1..dims[2].saturating_sub(1) {
        for iy in 1..dims[1].saturating_sub(1) {
            for ix in 1..dims[0].saturating_sub(1) {
                let v = val_at(ix, iy, iz);
                if v < min_value {
                    continue;
                }
                // Check 26-neighborhood
                let is_max = (-1i64..=1).all(|dz| {
                    (-1i64..=1).all(|dy| {
                        (-1i64..=1).all(|dx| {
                            if dx == 0 && dy == 0 && dz == 0 {
                                return true;
                            }
                            let nx = (ix as i64 + dx) as usize;
                            let ny = (iy as i64 + dy) as usize;
                            let nz = (iz as i64 + dz) as usize;
                            v >= val_at(nx, ny, nz)
                        })
                    })
                });
                if is_max {
                    pts.push([
                        origin[0] + ix as f64 * spacing[0],
                        origin[1] + iy as f64 * spacing[1],
                        origin[2] + iz as f64 * spacing[2],
                    ]);
                    strengths.push(v);
                }
            }
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Strength", strengths, 1,
        )));
    result
}

/// Detect edges in a 2D image using gradient magnitude threshold.
pub fn detect_edges_2d(image: &ImageData, array_name: &str, threshold: f64) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1];
    let mut buf = [0.0f64];
    let vals2d: Vec<f64> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();
    let val = |ix: usize, iy: usize| -> f64 {
        let idx = ix + iy * dims[0];
        if idx < vals2d.len() {
            vals2d[idx]
        } else {
            0.0
        }
    };
    let mut edges = vec![0.0f64; n];

    for iy in 1..dims[1].saturating_sub(1) {
        for ix in 1..dims[0].saturating_sub(1) {
            let gx = val(ix + 1, iy) - val(ix - 1, iy);
            let gy = val(ix, iy + 1) - val(ix, iy - 1);
            let mag = (gx * gx + gy * gy).sqrt();
            edges[ix + iy * dims[0]] = if mag >= threshold { mag } else { 0.0 };
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Edges", edges, 1)));
    result
}

/// Non-maximum suppression on a scalar field (thin detected features).
pub fn non_maximum_suppression_3d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut output = vec![0.0f64; n];

    for iz in 1..dims[2].saturating_sub(1) {
        for iy in 1..dims[1].saturating_sub(1) {
            for ix in 1..dims[0].saturating_sub(1) {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let v = vals[idx];
                let is_max = [
                    vals[idx - 1],
                    vals[idx + 1],
                    vals[idx - dims[0]],
                    vals[idx + dims[0]],
                    vals[idx - dims[0] * dims[1]],
                    vals[idx + dims[0] * dims[1]],
                ]
                .iter()
                .all(|&nb| v >= nb);
                output[idx] = if is_max { v } else { 0.0 };
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_blob() {
        let image = ImageData::from_function(
            [10, 10, 10],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, z| (-(((x - 5.0).powi(2) + (y - 5.0).powi(2) + (z - 5.0).powi(2)) / 2.0)).exp(),
        );
        let blobs = detect_blobs_3d(&image, "val", 0.5);
        assert!(blobs.points.len() >= 1);
    }
    #[test]
    fn edges_2d() {
        let image = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _, _| if x > 10.0 { 1.0 } else { 0.0 },
        );
        let result = detect_edges_2d(&image, "val", 0.1);
        assert!(result.point_data().get_array("Edges").is_some());
    }
    #[test]
    fn nms() {
        let image = ImageData::from_function(
            [8, 8, 8],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, z| (x - 4.0).powi(2) + (y - 4.0).powi(2) + (z - 4.0).powi(2),
        );
        let result = non_maximum_suppression_3d(&image, "val");
        assert!(result.point_data().get_array("val").is_some());
    }
}
