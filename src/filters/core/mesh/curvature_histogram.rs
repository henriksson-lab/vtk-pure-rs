use crate::data::PolyData;

/// Compute a curvature histogram for the mesh.
///
/// Returns (bin_centers, counts) for the Laplacian curvature magnitude distribution.
pub fn curvature_histogram(input: &PolyData, n_bins: usize) -> (Vec<f64>, Vec<usize>) {
    let n = input.points.len();
    if n == 0 {
        return (vec![], vec![]);
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut curvatures = Vec::new();
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = input.points.get(i);
        let cnt = neighbors[i].len() as f64;
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for &j in &neighbors[i] {
            let q = input.points.get(j);
            lx += q[0] - p[0];
            ly += q[1] - p[1];
            lz += q[2] - p[2];
        }
        curvatures.push((lx * lx + ly * ly + lz * lz).sqrt() / cnt);
    }

    if curvatures.is_empty() {
        return (vec![], vec![]);
    }
    let nb = n_bins.max(1);
    let max_c = curvatures.iter().copied().fold(0.0f64, f64::max).max(1e-15);
    let bw = max_c / nb as f64;

    let centers: Vec<f64> = (0..nb).map(|i| (i as f64 + 0.5) * bw).collect();
    let mut counts = vec![0usize; nb];
    for &c in &curvatures {
        let bin = (c / bw).floor() as usize;
        counts[bin.min(nb - 1)] += 1;
    }
    (centers, counts)
}

/// Compute curvature percentiles.
pub fn curvature_percentiles(input: &PolyData) -> [f64; 5] {
    // [min, p25, median, p75, max]
    let n = input.points.len();
    if n == 0 {
        return [0.0; 5];
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut curvatures: Vec<f64> = Vec::new();
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = input.points.get(i);
        let cnt = neighbors[i].len() as f64;
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for &j in &neighbors[i] {
            let q = input.points.get(j);
            lx += q[0] - p[0];
            ly += q[1] - p[1];
            lz += q[2] - p[2];
        }
        curvatures.push((lx * lx + ly * ly + lz * lz).sqrt() / cnt);
    }

    if curvatures.is_empty() {
        return [0.0; 5];
    }
    curvatures.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let nc = curvatures.len();
    [
        curvatures[0],
        curvatures[nc / 4],
        curvatures[nc / 2],
        curvatures[3 * nc / 4],
        curvatures[nc - 1],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_basic() {
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

        let (centers, counts) = curvature_histogram(&pd, 5);
        assert_eq!(centers.len(), 5);
        let total: usize = counts.iter().sum();
        assert!(total > 0);
    }

    #[test]
    fn percentiles() {
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

        let p = curvature_percentiles(&pd);
        assert!(p[0] <= p[2]);
        assert!(p[2] <= p[4]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (c, _) = curvature_histogram(&pd, 5);
        assert!(c.is_empty());
    }

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 99]);

        let (_, counts) = curvature_histogram(&pd, 3);
        assert_eq!(counts.iter().sum::<usize>(), 2);
        let p = curvature_percentiles(&pd);
        assert!(p[4].is_finite());
    }
}
