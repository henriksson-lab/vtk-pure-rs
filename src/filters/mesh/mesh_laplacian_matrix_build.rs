//! Build sparse Laplacian matrix representation.
use crate::data::PolyData;
pub struct SparseLaplacian {
    pub n: usize,
    pub rows: Vec<usize>,
    pub cols: Vec<usize>,
    pub vals: Vec<f64>,
}
pub fn build_uniform_laplacian(mesh: &PolyData) -> SparseLaplacian {
    let n = mesh.points.len();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        for i in 0..cell.len() {
            add_neighbor_pair(&mut nb, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_pair(&mut nb, n, edge[0], edge[1]);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_neighbor_pair(&mut nb, n, tri[0], tri[1]);
            add_neighbor_pair(&mut nb, n, tri[1], tri[2]);
            add_neighbor_pair(&mut nb, n, tri[2], tri[0]);
        }
    }
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut vals = Vec::new();
    for i in 0..n {
        let deg = nb[i].len() as f64;
        if deg < 1.0 {
            continue;
        }
        rows.push(i);
        cols.push(i);
        vals.push(deg);
        for &j in &nb[i] {
            rows.push(i);
            cols.push(j);
            vals.push(-1.0);
        }
    }
    SparseLaplacian {
        n,
        rows,
        cols,
        vals,
    }
}
pub fn apply_laplacian(lap: &SparseLaplacian, x: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; lap.n];
    for k in 0..lap.rows.len() {
        result[lap.rows[k]] += lap.vals[k] * x[lap.cols[k]];
    }
    result
}
pub fn laplacian_nnz(mesh: &PolyData) -> usize {
    let lap = build_uniform_laplacian(mesh);
    lap.rows.len()
}
fn add_neighbor_pair(nb: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_build() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let lap = build_uniform_laplacian(&m);
        assert!(lap.rows.len() > 4);
        assert_eq!(lap.n, 4);
    }
    #[test]
    fn test_apply() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let lap = build_uniform_laplacian(&m);
        let x = vec![1.0, 1.0, 1.0];
        let lx = apply_laplacian(&lap, &x);
        for v in &lx {
            assert!(v.abs() < 1e-10);
        }
    } // constant -> zero Laplacian
}
