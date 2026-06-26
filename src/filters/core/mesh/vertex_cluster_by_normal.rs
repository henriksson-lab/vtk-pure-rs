use crate::data::{AnyDataArray, DataArray, PolyData};

/// Cluster vertices by their normal direction using k-means.
///
/// Reads the specified normals array (3-component) from point data, then
/// runs k-means clustering on the unit-sphere normal directions.
/// Adds a "NormalCluster" (1-component i32) array to point data.
///
/// The `seed` parameter controls the deterministic initialization of centroids.
pub fn cluster_by_normal(input: &PolyData, k: usize, normals_array: &str, seed: u64) -> PolyData {
    let arr = match input.point_data().get_array(normals_array) {
        Some(a) => a,
        None => return input.clone(),
    };
    if arr.num_components() != 3 || arr.num_tuples() != input.points.len() {
        return input.clone();
    }

    let n = arr.num_tuples();
    if n == 0 || k == 0 {
        return input.clone();
    }
    let k = k.min(n);

    // Read normals and normalize them
    let mut normals = Vec::with_capacity(n);
    let mut buf = [0.0f64; 3];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let len: f64 = (buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2]).sqrt();
        if len > 1e-20 {
            normals.push([buf[0] / len, buf[1] / len, buf[2] / len]);
        } else {
            normals.push([0.0, 0.0, 1.0]);
        }
    }

    // Initialize centroids deterministically using seed-based selection
    let mut centroids = Vec::with_capacity(k);
    let mut rng_state: u64 = seed;
    for i in 0..k {
        // Simple hash-like index selection
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx: usize = ((rng_state >> 33) as usize + i) % n;
        centroids.push(normals[idx]);
    }

    let mut labels = vec![0i32; n];
    let max_iters: u32 = 50;

    for _ in 0..max_iters {
        // Assignment step: assign each normal to nearest centroid
        let mut changed: bool = false;
        for i in 0..n {
            let mut best_k: usize = 0;
            let mut best_dist: f64 = f64::MAX;
            for j in 0..k {
                // Use 1 - dot product as distance (angular distance proxy on unit sphere)
                let d: f64 = 1.0 - dot(&normals[i], &centroids[j]);
                if d < best_dist {
                    best_dist = d;
                    best_k = j;
                }
            }
            let new_label: i32 = best_k as i32;
            if labels[i] != new_label {
                labels[i] = new_label;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Update step: recompute centroids as mean of assigned normals, then normalize
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];
        for i in 0..n {
            let c = labels[i] as usize;
            sums[c][0] += normals[i][0];
            sums[c][1] += normals[i][1];
            sums[c][2] += normals[i][2];
            counts[c] += 1;
        }
        for j in 0..k {
            if counts[j] > 0 {
                let len: f64 =
                    (sums[j][0] * sums[j][0] + sums[j][1] * sums[j][1] + sums[j][2] * sums[j][2])
                        .sqrt();
                if len > 1e-20 {
                    centroids[j] = [sums[j][0] / len, sums[j][1] / len, sums[j][2] / len];
                }
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::I32(DataArray::from_vec(
            "NormalCluster",
            labels,
            1,
        )));
    pd
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_data() -> PolyData {
        // 6 points with normals pointing in 3 distinct directions (+x, +y, +z)
        let mut pd = PolyData::new();
        for _ in 0..6 {
            pd.points.push([0.0, 0.0, 0.0]);
        }
        let normals_data: Vec<f64> = vec![
            1.0, 0.0, 0.0, // +x
            0.99, 0.01, 0.0, // ~+x
            0.0, 1.0, 0.0, // +y
            0.01, 0.99, 0.0, // ~+y
            0.0, 0.0, 1.0, // +z
            0.0, 0.01, 0.99, // ~+z
        ];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                normals_data,
                3,
            )));
        pd
    }

    #[test]
    fn clusters_distinct_directions() {
        let pd = make_test_data();
        let result = cluster_by_normal(&pd, 3, "Normals", 42);
        let arr = result.point_data().get_array("NormalCluster").unwrap();
        assert_eq!(arr.num_tuples(), 6);

        // Points 0 and 1 should be in the same cluster
        let mut v = [0.0f64];
        arr.tuple_as_f64(0, &mut v);
        let c0: i32 = v[0] as i32;
        arr.tuple_as_f64(1, &mut v);
        let c1: i32 = v[0] as i32;
        assert_eq!(c0, c1, "similar normals should cluster together");

        // Points 2 and 3 should be in the same cluster
        arr.tuple_as_f64(2, &mut v);
        let c2: i32 = v[0] as i32;
        arr.tuple_as_f64(3, &mut v);
        let c3: i32 = v[0] as i32;
        assert_eq!(c2, c3, "similar normals should cluster together");

        // The three groups should be in different clusters
        assert_ne!(c0, c2, "+x and +y should be in different clusters");
    }

    #[test]
    fn missing_array_returns_clone() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = cluster_by_normal(&pd, 2, "NonExistent", 0);
        assert!(result.point_data().get_array("NormalCluster").is_none());
    }

    #[test]
    fn invalid_normals_array_returns_clone() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0],
                1,
            )));
        let result = cluster_by_normal(&pd, 1, "Normals", 0);
        assert!(result.point_data().get_array("NormalCluster").is_none());
    }

    #[test]
    fn single_cluster() {
        let pd = make_test_data();
        let result = cluster_by_normal(&pd, 1, "Normals", 42);
        let arr = result.point_data().get_array("NormalCluster").unwrap();
        // With k=1, all should be cluster 0
        let mut v = [0.0f64];
        for i in 0..6 {
            arr.tuple_as_f64(i, &mut v);
            assert_eq!(v[0] as i32, 0, "all should be cluster 0 with k=1");
        }
    }
}
