//! Uniform mesh coarsening by grid-based vertex clustering.
use crate::data::{CellArray, Points, PolyData};
pub fn coarsen_by_ratio(mesh: &PolyData, ratio: f64) -> PolyData {
    let n = mesh.points.len();
    if n < 4 || ratio >= 1.0 {
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
    let diag = ((mx[0] - mn[0]).powi(2) + (mx[1] - mn[1]).powi(2) + (mx[2] - mn[2]).powi(2)).sqrt();
    let target_verts = (n as f64 * ratio.clamp(0.01, 1.0)) as usize;
    let cell_size = diag / (target_verts as f64).cbrt().max(1.0);
    grid_simplify(mesh, cell_size)
}
fn grid_simplify(mesh: &PolyData, cs: f64) -> PolyData {
    let n = mesh.points.len();
    let cs = cs.max(1e-15);
    let mut grid: std::collections::HashMap<(i64, i64, i64), ([f64; 3], usize)> =
        std::collections::HashMap::new();
    let mut keys = Vec::new();
    let mut remap = vec![0usize; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        let k = (
            (p[0] / cs).floor() as i64,
            (p[1] / cs).floor() as i64,
            (p[2] / cs).floor() as i64,
        );
        let e = grid.entry(k).or_insert_with(|| {
            keys.push(k);
            ([0.0, 0.0, 0.0], 0)
        });
        e.0[0] += p[0];
        e.0[1] += p[1];
        e.0[2] += p[2];
        e.1 += 1;
    }
    let mut pts = Points::<f64>::new();
    let mut key_to_idx: std::collections::HashMap<(i64, i64, i64), usize> =
        std::collections::HashMap::new();
    for &k in &keys {
        let v = &grid[&k];
        let c = v.1 as f64;
        let idx = pts.len();
        pts.push([v.0[0] / c, v.0[1] / c, v.0[2] / c]);
        key_to_idx.insert(k, idx);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        let k = (
            (p[0] / cs).floor() as i64,
            (p[1] / cs).floor() as i64,
            (p[2] / cs).floor() as i64,
        );
        remap[i] = key_to_idx[&k];
    }
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.iter().any(|&v| v < 0 || v as usize >= remap.len()) {
            continue;
        }
        let mut mapped = Vec::with_capacity(cell.len());
        for &v in cell {
            let id = remap[v as usize] as i64;
            if mapped.last().copied() != Some(id) {
                mapped.push(id);
            }
        }
        if mapped.len() > 1 && mapped.first() == mapped.last() {
            mapped.pop();
        }
        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= 3 && unique.len() == mapped.len() {
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
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2], [2, 3, 5]],
        );
        let r = coarsen_by_ratio(&m, 0.5);
        assert!(r.points.len() <= 6);
    }
}
