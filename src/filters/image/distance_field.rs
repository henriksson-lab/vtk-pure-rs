//! Euclidean distance field computation for binary ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the squared Euclidean distance transform of a binary image.
///
/// Like `vtkImageEuclideanDistance` with initialization enabled, zero-valued
/// voxels are distance sources and non-zero voxels receive the squared distance
/// to the nearest source voxel.
pub fn distance_transform(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let spacing = image.spacing();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];

    let max_dist = i32::MAX as f64;
    let mut dist = vec![max_dist; n];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] == 0.0 {
            dist[i] = 0.0;
        }
    }

    saito_axis(&mut dist, dims, 0, spacing[0] * spacing[0], max_dist);
    saito_axis(&mut dist, dims, 1, spacing[1] * spacing[1], max_dist);
    saito_axis(&mut dist, dims, 2, spacing[2] * spacing[2], max_dist);

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

fn saito_axis(dist: &mut [f64], dims: [usize; 3], axis: usize, spacing2: f64, max_dist: f64) {
    let line_count = match axis {
        0 => dims[1] * dims[2],
        1 => dims[0] * dims[2],
        _ => dims[0] * dims[1],
    };
    let line_len = dims[axis];
    if line_len == 0 {
        return;
    }

    let mut sq = vec![max_dist; line_len * 2 + 2];
    for df in 0..=line_len {
        sq[df] = df as f64 * df as f64 * spacing2;
    }

    let mut line_values = vec![0.0; line_len];
    for line in 0..line_count {
        for q in 0..line_len {
            let idx = line_index(line, q, dims, axis);
            line_values[q] = dist[idx];
        }

        if axis == 0 {
            saito_first_pass(&mut line_values, &sq);
        } else {
            saito_later_pass(&mut line_values, &sq, spacing2);
        }

        for q in 0..line_len {
            let idx = line_index(line, q, dims, axis);
            dist[idx] = line_values[q];
        }
    }
}

fn saito_first_pass(values: &mut [f64], sq: &[f64]) {
    let line_len = values.len();

    let mut df = line_len;
    for value in values.iter_mut() {
        if *value != 0.0 {
            df += 1;
            *value = (*value).min(sq[df]);
        } else {
            df = 0;
        }
    }

    df = line_len;
    for value in values.iter_mut().rev() {
        if *value != 0.0 {
            df += 1;
            *value = (*value).min(sq[df]);
        } else {
            df = 0;
        }
    }
}

fn saito_later_pass(values: &mut [f64], sq: &[f64], spacing2: f64) {
    let line_len = values.len();
    let buffered = values.to_vec();

    let mut a = 0usize;
    let mut buffer = buffered[0];
    for q in 1..line_len {
        a = a.saturating_sub(1);
        if buffered[q] > buffer + sq[1] {
            let mut b = ((((buffered[q] - buffer) / spacing2) - 1.0) / 2.0).floor() as usize;
            b = b.min(line_len - 1 - q);
            for n in a..=b {
                let m = buffer + sq[n + 1];
                if buffered[q + n] <= m {
                    break;
                }
                if m < values[q + n] {
                    values[q + n] = m;
                }
            }
            a = b;
        } else {
            a = 0;
        }
        buffer = buffered[q];
    }

    a = 0;
    buffer = buffered[line_len - 1];
    for q in (0..line_len - 1).rev() {
        a = a.saturating_sub(1);
        if buffered[q] > buffer + sq[1] {
            let mut b = ((((buffered[q] - buffer) / spacing2) - 1.0) / 2.0).floor() as usize;
            b = b.min(q);
            for n in a..=b {
                let m = buffer + sq[n + 1];
                if buffered[q - n] <= m {
                    break;
                }
                if m < values[q - n] {
                    values[q - n] = m;
                }
            }
            a = b;
        } else {
            a = 0;
        }
        buffer = buffered[q];
    }
}

fn line_index(line: usize, q: usize, dims: [usize; 3], axis: usize) -> usize {
    let nx = dims[0];
    let ny = dims[1];
    match axis {
        0 => {
            let j = line % ny;
            let k = line / ny;
            q + j * nx + k * nx * ny
        }
        1 => {
            let i = line % nx;
            let k = line / nx;
            i + q * nx + k * nx * ny
        }
        _ => {
            let i = line % nx;
            let j = line / nx;
            i + j * nx + q * nx * ny
        }
    }
}

/// Compute signed distance field: negative inside foreground, positive outside.
pub fn signed_distance_transform(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];

    let inv_vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                0.0
            } else {
                1.0
            }
        })
        .collect();

    let inside = distance_transform(image, array_name);
    let mut inv_image = image.clone();
    inv_image
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("_inv", inv_vals, 1)));
    let outside = distance_transform(&inv_image, "_inv");

    let in_arr = inside.point_data().get_array("DistanceField").unwrap();
    let out_arr = outside.point_data().get_array("DistanceField").unwrap();

    let mut sdf = Vec::with_capacity(n);
    let mut ob = [0.0f64];
    let mut ib = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        out_arr.tuple_as_f64(i, &mut ob);
        in_arr.tuple_as_f64(i, &mut ib);
        sdf.push(if buf[0] > 0.5 {
            -ib[0].max(0.0).sqrt()
        } else {
            ob[0].max(0.0).sqrt()
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
        // Non-zero pixels receive squared distance to the nearest zero pixel.
        arr.tuple_as_f64(10 + 10 * 20, &mut buf);
        assert!(buf[0] > 0.0);
        // Zero pixels are distance sources.
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
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

    #[test]
    fn signed_rejects_vector_arrays_without_panicking() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![0.0, 1.0, 1.0, 0.0],
                2,
            )));

        let result = signed_distance_transform(&img, "mask");
        assert!(result.point_data().get_array("SignedDistance").is_none());
        assert_eq!(
            result
                .point_data()
                .get_array("mask")
                .unwrap()
                .num_components(),
            2
        );
    }
}
