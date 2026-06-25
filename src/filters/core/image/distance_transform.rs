use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute an approximate Euclidean distance transform on a binary ImageData.
///
/// For each voxel, computes the distance to the nearest voxel where scalar >= threshold.
/// Uses a two-pass chamfer distance approximation over the full 3x3x3 neighborhood.
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

    let mut forward_offsets: Vec<(isize, isize, isize, f64)> = Vec::new();
    let mut backward_offsets: Vec<(isize, isize, isize, f64)> = Vec::new();
    for dz in -1isize..=1 {
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 && dz == 0 {
                    continue;
                }
                let cost = ((dx as f64 * spacing[0]).powi(2)
                    + (dy as f64 * spacing[1]).powi(2)
                    + (dz as f64 * spacing[2]).powi(2))
                .sqrt();
                if dz < 0 || (dz == 0 && dy < 0) || (dz == 0 && dy == 0 && dx < 0) {
                    forward_offsets.push((dx, dy, dz, cost));
                } else {
                    backward_offsets.push((dx, dy, dz, cost));
                }
            }
        }
    }

    // Forward pass
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let cur = idx(i, j, k);
                for &(dx, dy, dz, cost) in &forward_offsets {
                    let x = i as isize + dx;
                    let y = j as isize + dy;
                    let z = k as isize + dz;
                    if x >= 0
                        && x < nx as isize
                        && y >= 0
                        && y < ny as isize
                        && z >= 0
                        && z < nz as isize
                    {
                        let prev = idx(x as usize, y as usize, z as usize);
                        dist[cur] = dist[cur].min(dist[prev] + cost);
                    }
                }
            }
        }
    }

    // Backward pass
    for k in (0..nz).rev() {
        for j in (0..ny).rev() {
            for i in (0..nx).rev() {
                let cur = idx(i, j, k);
                for &(dx, dy, dz, cost) in &backward_offsets {
                    let x = i as isize + dx;
                    let y = j as isize + dy;
                    let z = k as isize + dz;
                    if x >= 0
                        && x < nx as isize
                        && y >= 0
                        && y < ny as isize
                        && z >= 0
                        && z < nz as isize
                    {
                        let prev = idx(x as usize, y as usize, z as usize);
                        dist[cur] = dist[cur].min(dist[prev] + cost);
                    }
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
    fn diagonal_neighbor_uses_euclidean_step() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let mut values = vec![0.0f64; 9];
        values[0] = 1.0;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));

        let result = image_distance_transform(&img, "mask", 0.5);
        let arr = result.point_data().get_array("DistanceTransform").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 2.0f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_distance_transform(&img, "nope", 0.5);
        assert!(result.point_data().get_array("DistanceTransform").is_none());
    }
}
