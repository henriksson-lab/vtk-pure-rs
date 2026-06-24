use crate::data::PolyData;

/// Compute the Dirichlet energy of the mesh (sum of squared edge lengths).
///
/// Lower energy = smoother mesh. Useful for comparing mesh quality.
pub fn dirichlet_energy(input: &PolyData) -> f64 {
    let mut energy = 0.0;
    let mut seen = std::collections::HashSet::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa = input.points.get(a);
                let pb = input.points.get(b);
                energy +=
                    (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);
            }
        }
    }
    energy
}

/// Compute the Willmore energy (integral of squared mean curvature).
///
/// Approximated as sum of squared Laplacian magnitudes over the mesh.
pub fn willmore_energy(input: &PolyData) -> f64 {
    let n = input.points.len();
    if n == 0 {
        return 0.0;
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

    let mut energy = 0.0;
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
        lx /= cnt;
        ly /= cnt;
        lz /= cnt;
        energy += lx * lx + ly * ly + lz * lz;
    }
    energy
}

/// Compute the ratio of Willmore to Dirichlet energy (smoothness metric).
pub fn smoothness_ratio(input: &PolyData) -> f64 {
    let d = dirichlet_energy(input);
    let w = willmore_energy(input);
    if d > 1e-15 {
        w / d
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_low_willmore() {
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

        let w = willmore_energy(&pd);
        // Flat mesh has low curvature energy (boundary vertices contribute some)
        let d = dirichlet_energy(&pd);
        let ratio = if d > 0.0 { w / d } else { 0.0 };
        assert!(ratio < 1.0, "ratio={}", ratio);
    }

    #[test]
    fn dirichlet_positive() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert!(dirichlet_energy(&pd) > 0.0);
    }

    #[test]
    fn smoothness_bounded() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let r = smoothness_ratio(&pd);
        assert!(r >= 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(dirichlet_energy(&pd), 0.0);
        assert_eq!(willmore_energy(&pd), 0.0);
    }
}
