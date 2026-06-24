//! Resample points uniformly on mesh surface using face area weighting.
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn resample_uniform(mesh: &PolyData, target_count: usize, seed: u64) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect();
    if cells.is_empty() {
        return PolyData::new();
    }
    let areas: Vec<f64> = cells
        .iter()
        .map(|c| {
            let a = mesh.points.get(c[0] as usize);
            let mut ta = 0.0;
            for i in 1..c.len() - 1 {
                let b = mesh.points.get(c[i] as usize);
                let cc = mesh.points.get(c[i + 1] as usize);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
                ta += 0.5
                    * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                    .sqrt();
            }
            ta
        })
        .collect();
    let total: f64 = areas.iter().sum();
    if total < 1e-30 {
        return PolyData::new();
    }
    let mut cum = Vec::with_capacity(areas.len());
    let mut acc = 0.0;
    for &a in &areas {
        acc += a / total;
        cum.push(acc);
    }
    let mut rng = seed;
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for _ in 0..target_count {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = ((rng >> 33) as f64) / (u32::MAX as f64);
        let ci = cum.partition_point(|&c| c < r).min(cells.len() - 1);
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut u = ((rng >> 33) as f64) / (u32::MAX as f64);
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut v = ((rng >> 33) as f64) / (u32::MAX as f64);
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;
        let a = mesh.points.get(cells[ci][0] as usize);
        let b = mesh.points.get(cells[ci][1] as usize);
        let c = mesh
            .points
            .get(cells[ci][2.min(cells[ci].len() - 1)] as usize);
        let idx = pts.len();
        pts.push([
            a[0] * w + b[0] * u + c[0] * v,
            a[1] * w + b[1] * u + c[1] * v,
            a[2] * w + b[2] * u + c[2] * v,
        ]);
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
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
        let r = resample_uniform(&m, 50, 42);
        assert_eq!(r.points.len(), 50);
    }
}
