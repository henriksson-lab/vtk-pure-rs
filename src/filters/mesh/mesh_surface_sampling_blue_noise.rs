//! Blue noise surface sampling (dart-throwing on mesh).
use crate::data::{CellArray, Points, PolyData};
pub fn blue_noise_sample(
    mesh: &PolyData,
    min_distance: f64,
    max_attempts: usize,
    seed: u64,
) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect();
    if cells.is_empty() {
        return PolyData::new();
    }
    // Compute cumulative area for area-weighted sampling
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
    let min_d2 = min_distance * min_distance;
    let mut rng = seed;
    let mut samples: Vec<[f64; 3]> = Vec::new();
    let next_f = |rng: &mut u64| -> f64 {
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*rng >> 33) as f64) / (u32::MAX as f64)
    };
    for _ in 0..max_attempts {
        let r = next_f(&mut rng);
        let ci = cum.partition_point(|&c| c < r).min(cells.len() - 1);
        let mut u = next_f(&mut rng);
        let mut v = next_f(&mut rng);
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
        let p = [
            a[0] * w + b[0] * u + c[0] * v,
            a[1] * w + b[1] * u + c[1] * v,
            a[2] * w + b[2] * u + c[2] * v,
        ];
        // Check distance to all existing samples
        let too_close = samples.iter().any(|s| {
            (s[0] - p[0]).powi(2) + (s[1] - p[1]).powi(2) + (s[2] - p[2]).powi(2) < min_d2
        });
        if !too_close {
            samples.push(p);
        }
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for (i, s) in samples.iter().enumerate() {
        pts.push(*s);
        verts.push_cell(&[i as i64]);
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
        let r = blue_noise_sample(&m, 1.0, 1000, 42);
        assert!(r.points.len() > 5); // should fit many samples
                                     // Check minimum distance
        for i in 0..r.points.len() {
            for j in i + 1..r.points.len() {
                let a = r.points.get(i);
                let b = r.points.get(j);
                let d =
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                assert!(d >= 0.99, "samples too close: {d}");
            }
        }
    }
}
