//! Cluster vertices using a uniform grid (voxel-based simplification).
use crate::data::{CellArray, Points, PolyData};
pub fn grid_cluster(mesh: &PolyData, cell_size: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let cs = cell_size.max(1e-15);
    let mut grid: std::collections::HashMap<(i64, i64, i64), (usize, [f64; 3], usize)> =
        std::collections::HashMap::new();
    let mut remap = vec![0usize; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        let gx = (p[0] / cs).floor() as i64;
        let gy = (p[1] / cs).floor() as i64;
        let gz = (p[2] / cs).floor() as i64;
        let e = grid.entry((gx, gy, gz)).or_insert((0, [0.0, 0.0, 0.0], 0));
        e.1[0] += p[0];
        e.1[1] += p[1];
        e.1[2] += p[2];
        e.2 += 1;
        remap[i] = 0; // placeholder
    }
    let mut pts = Points::<f64>::new();
    let mut cell_to_idx: std::collections::HashMap<(i64, i64, i64), usize> =
        std::collections::HashMap::new();
    for (&k, v) in &grid {
        let c = v.2 as f64;
        let idx = pts.len();
        pts.push([v.1[0] / c, v.1[1] / c, v.1[2] / c]);
        cell_to_idx.insert(k, idx);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let gx = (p[0] / cs).floor() as i64;
        let gy = (p[1] / cs).floor() as i64;
        let gz = (p[2] / cs).floor() as i64;
        remap[i] = cell_to_idx[&(gx, gy, gz)];
    }
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let mapped: Vec<i64> = cell.iter().map(|&v| remap[v as usize] as i64).collect();
        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= 3 {
            polys.push_cell(&mapped);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = grid_cluster(&m, 3.0);
        assert!(r.points.len() <= 3);
        assert!(r.polys.num_cells() <= 1);
    }
}
