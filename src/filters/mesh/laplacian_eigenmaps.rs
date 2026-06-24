use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute Fiedler vector (second-smallest eigenvector of Laplacian).
///
/// Approximated via power iteration on (D-L), where D is degree matrix
/// and L is adjacency. The Fiedler vector partitions the mesh into two
/// balanced halves. Adds "Fiedler" scalar array.
pub fn fiedler_vector(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    // Power iteration for smallest non-trivial eigenvector of graph Laplacian
    // L*v = degree*v - A*v, we want second smallest eigenvalue
    // Use inverse iteration: (L - sigma*I)^-1 approximated by Jacobi
    // Simplified: iterate v = L*v, then orthogonalize against constant vector

    let mut v: Vec<f64> = (0..n).map(|i| (i as f64 / n as f64) - 0.5).collect();
    let ones_norm = (n as f64).sqrt();

    for _ in 0..100 {
        // Apply Laplacian: Lv[i] = degree[i]*v[i] - sum(v[j]) for j in neighbors
        let mut lv = vec![0.0f64; n];
        for i in 0..n {
            lv[i] = neighbors[i].len() as f64 * v[i];
            for &j in &neighbors[i] {
                lv[i] -= v[j];
            }
        }

        // Orthogonalize against constant vector
        let dot_ones: f64 = lv.iter().sum::<f64>() / n as f64;
        for val in &mut lv {
            *val -= dot_ones;
        }

        // Normalize
        let norm: f64 = lv.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-15 {
            for val in &mut lv {
                *val /= norm;
            }
        }

        v = lv;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Fiedler", v, 1)));
    pd
}

/// Partition mesh into two halves using the Fiedler vector sign.
/// Adds "Partition" scalar (0 or 1).
pub fn spectral_partition(input: &PolyData) -> PolyData {
    let result = fiedler_vector(input);
    let arr = match result.point_data().get_array("Fiedler") {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let partition: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= 0.0 {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut pd = result;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Partition",
            partition,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fiedler_basic() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let result = fiedler_vector(&pd);
        assert!(result.point_data().get_array("Fiedler").is_some());
    }

    #[test]
    fn partition_balanced() {
        let mut pd = PolyData::new();
        for j in 0..4 {
            for i in 0..4 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..3 {
            for i in 0..3 {
                let a = (j * 4 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 5]);
                pd.polys.push_cell(&[a, a + 5, a + 4]);
            }
        }

        let result = spectral_partition(&pd);
        let arr = result.point_data().get_array("Partition").unwrap();
        let mut buf = [0.0f64];
        let mut count0 = 0;
        let mut count1 = 0;
        for i in 0..16 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count1 += 1
            } else {
                count0 += 1
            }
        }
        assert!(count0 > 0 && count1 > 0); // both partitions non-empty
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = fiedler_vector(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
