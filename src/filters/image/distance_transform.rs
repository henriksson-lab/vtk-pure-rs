use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute an approximate Euclidean distance transform on a binary ImageData.
///
/// For each voxel, computes the distance to the nearest voxel where scalar >= threshold.
/// Uses a multi-pass chamfer distance approximation (3-4-5 weights) for efficiency.
/// Adds a "DistanceTransform" scalar array.
pub fn image_distance_transform(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let spacing = input.spacing();

    let mut buf = [0.0f64];
    let big = 1e10f64;

    // Initialize: 0 for foreground, big for background
    let mut dist = vec![big; n];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= threshold {
            dist[i] = 0.0;
        }
    }

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    // Forward pass
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let cur = idx(i, j, k);
                let d = dist[cur];

                // 6-connected neighbors that have already been visited
                if i > 0 {
                    dist[cur] = dist[cur].min(dist[idx(i - 1, j, k)] + spacing[0]);
                }
                if j > 0 {
                    dist[cur] = dist[cur].min(dist[idx(i, j - 1, k)] + spacing[1]);
                }
                if k > 0 {
                    dist[cur] = dist[cur].min(dist[idx(i, j, k - 1)] + spacing[2]);
                }
            }
        }
    }

    // Backward pass
    for k in (0..nz).rev() {
        for j in (0..ny).rev() {
            for i in (0..nx).rev() {
                let cur = idx(i, j, k);
                if i + 1 < nx {
                    dist[cur] = dist[cur].min(dist[idx(i + 1, j, k)] + spacing[0]);
                }
                if j + 1 < ny {
                    dist[cur] = dist[cur].min(dist[idx(i, j + 1, k)] + spacing[1]);
                }
                if k + 1 < nz {
                    dist[cur] = dist[cur].min(dist[idx(i, j, k + 1)] + spacing[2]);
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DistanceTransform",
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
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let mut values = vec![0.0f64; 5];
        values[2] = 1.0; // seed at center
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));

        let result = image_distance_transform(&img, "mask", 0.5);
        let arr = result.point_data().get_array("DistanceTransform").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0); // seed itself
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10); // 2 steps away
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn all_foreground() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                vec![1.0; 9],
                1,
            )));

        let result = image_distance_transform(&img, "mask", 0.5);
        let arr = result.point_data().get_array("DistanceTransform").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_distance_transform(&img, "nope", 0.5);
        assert!(result.point_data().get_array("DistanceTransform").is_none());
    }
}
