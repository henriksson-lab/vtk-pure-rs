//! Octree-based vertex clustering for LOD generation.
use crate::data::{CellArray, Points, PolyData};
pub fn octree_cluster(mesh: &PolyData, max_depth: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    let size = [
        (mx[0] - mn[0]).max(1e-15),
        (mx[1] - mn[1]).max(1e-15),
        (mx[2] - mn[2]).max(1e-15),
    ];
    let depth = max_depth.min(8);
    let cells_per_axis = 1usize << depth;
    let cs = [
        size[0] / cells_per_axis as f64,
        size[1] / cells_per_axis as f64,
        size[2] / cells_per_axis as f64,
    ];
    let mut grid: std::collections::HashMap<(usize, usize, usize), ([f64; 3], usize)> =
        std::collections::HashMap::new();
    let mut remap = vec![0usize; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        let gx = ((p[0] - mn[0]) / cs[0]).floor() as usize;
        let gy = ((p[1] - mn[1]) / cs[1]).floor() as usize;
        let gz = ((p[2] - mn[2]) / cs[2]).floor() as usize;
        let e = grid.entry((gx, gy, gz)).or_insert(([0.0, 0.0, 0.0], 0));
        e.0[0] += p[0];
        e.0[1] += p[1];
        e.0[2] += p[2];
        e.1 += 1;
    }
    let mut pts = Points::<f64>::new();
    let mut key_to_idx: std::collections::HashMap<(usize, usize, usize), usize> =
        std::collections::HashMap::new();
    for (&k, v) in &grid {
        let c = v.1 as f64;
        let idx = pts.len();
        pts.push([v.0[0] / c, v.0[1] / c, v.0[2] / c]);
        key_to_idx.insert(k, idx);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let gx = ((p[0] - mn[0]) / cs[0]).floor() as usize;
        let gy = ((p[1] - mn[1]) / cs[1]).floor() as usize;
        let gz = ((p[2] - mn[2]) / cs[2]).floor() as usize;
        remap[i] = key_to_idx[&(gx, gy, gz)];
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
        let r = octree_cluster(&m, 2);
        assert!(r.points.len() <= 3);
    }
}
