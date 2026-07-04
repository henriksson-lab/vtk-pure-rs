//! Blue noise surface sampling (dart-throwing on mesh).
use crate::data::{CellArray, Points, PolyData};
pub fn blue_noise_sample(
    mesh: &PolyData,
    min_distance: f64,
    max_attempts: usize,
    seed: u64,
) -> PolyData {
    let mut tris = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !valid_point_cell(mesh, cell) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            tris.push([cell[0] as usize, cell[i] as usize, cell[i + 1] as usize]);
        }
    }
    if tris.is_empty() {
        return PolyData::new();
    }
    // Compute cumulative area for area-weighted sampling
    let areas: Vec<f64> = tris
        .iter()
        .map(|tri| {
            let a = mesh.points.get(tri[0]);
            let b = mesh.points.get(tri[1]);
            let c = mesh.points.get(tri[2]);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt()
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
        ((*rng >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    };
    for _ in 0..max_attempts {
        let r = next_f(&mut rng);
        let ti = cum.partition_point(|&c| c < r).min(tris.len() - 1);
        let mut u = next_f(&mut rng);
        let mut v = next_f(&mut rng);
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;
        let tri = tris[ti];
        let a = mesh.points.get(tri[0]);
        let b = mesh.points.get(tri[1]);
        let c = mesh.points.get(tri[2]);
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

fn valid_point_cell(mesh: &PolyData, cell: &[i64]) -> bool {
    cell.iter()
        .all(|&id| id >= 0 && (id as usize) < mesh.points.len())
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
