use crate::data::{AnyDataArray, DataArray, PolyData};

/// K-means clustering of mesh vertices in 3D space.
///
/// Iteratively assigns vertices to nearest centroid and recomputes centroids.
/// Adds "KMeansCluster" scalar array.
pub fn kmeans_cluster(input: &PolyData, k: usize, iterations: usize, seed: u64) -> PolyData {
    let n = input.points.len();
    if n == 0 || k == 0 {
        return input.clone();
    }
    let k = k.min(n);

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Initialize centroids: first k points (deterministic)
    let mut rng = seed;
    let mut centroids: Vec<[f64; 3]> = Vec::with_capacity(k);
    let mut used = std::collections::HashSet::new();
    for _ in 0..k {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (rng >> 33) as usize % n;
        if !used.insert(idx) && centroids.len() < k {
            // fallback to next unused
            for j in 0..n {
                if used.insert(j) {
                    centroids.push(pts[j]);
                    break;
                }
            }
        } else {
            centroids.push(pts[idx]);
        }
    }
    // Fill remaining if needed
    while centroids.len() < k {
        centroids.push(pts[centroids.len() % n]);
    }

    let mut labels = vec![0usize; n];

    for _ in 0..iterations {
        // Assign
        for i in 0..n {
            let p = pts[i];
            let mut best = 0;
            let mut best_d = f64::MAX;
            for (ci, c) in centroids.iter().enumerate() {
                let d = (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = ci;
                }
            }
            labels[i] = best;
        }

        // Update centroids
        let mut sums = vec![[0.0f64; 3]; k];
        let mut counts = vec![0usize; k];
        for i in 0..n {
            let l = labels[i];
            sums[l][0] += pts[i][0];
            sums[l][1] += pts[i][1];
            sums[l][2] += pts[i][2];
            counts[l] += 1;
        }
        for ci in 0..k {
            if counts[ci] > 0 {
                let c = counts[ci] as f64;
                centroids[ci] = [sums[ci][0] / c, sums[ci][1] / c, sums[ci][2] / c];
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "KMeansCluster",
            labels_f,
            1,
        )));
    pd
}

/// Compute k-means inertia (sum of squared distances to centroids).
pub fn kmeans_inertia(input: &PolyData, k: usize, iterations: usize) -> f64 {
    let result = kmeans_cluster(input, k, iterations, 42);
    let arr = match result.point_data().get_array("KMeansCluster") {
        Some(a) => a,
        None => return 0.0,
    };
    let n = input.points.len();

    // Recompute centroids
    let mut buf = [0.0f64];
    let mut sums = vec![[0.0f64; 3]; k];
    let mut counts = vec![0usize; k];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let l = buf[0] as usize;
        let p = input.points.get(i);
        sums[l][0] += p[0];
        sums[l][1] += p[1];
        sums[l][2] += p[2];
        counts[l] += 1;
    }
    let centroids: Vec<[f64; 3]> = (0..k)
        .map(|ci| {
            if counts[ci] > 0 {
                let c = counts[ci] as f64;
                [sums[ci][0] / c, sums[ci][1] / c, sums[ci][2] / c]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    let mut inertia = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let l = buf[0] as usize;
        let p = input.points.get(i);
        let c = centroids[l];
        inertia += (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
    }
    inertia
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clusters() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        for i in 0..5 {
            pd.points.push([100.0 + i as f64, 0.0, 0.0]);
        }

        let result = kmeans_cluster(&pd, 2, 10, 42);
        let arr = result.point_data().get_array("KMeansCluster").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0];
        arr.tuple_as_f64(9, &mut buf);
        let c9 = buf[0];
        assert_ne!(c0, c9); // different clusters
    }

    #[test]
    fn inertia_decreases_with_k() {
        let mut pd = PolyData::new();
        for i in 0..20 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }

        let i1 = kmeans_inertia(&pd, 1, 10);
        let i2 = kmeans_inertia(&pd, 2, 10);
        assert!(i2 < i1); // more clusters = lower inertia
    }

    #[test]
    fn single_cluster() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = kmeans_cluster(&pd, 1, 5, 0);
        let arr = result.point_data().get_array("KMeansCluster").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = kmeans_cluster(&pd, 3, 5, 0);
        assert_eq!(result.points.len(), 0);
    }
}
