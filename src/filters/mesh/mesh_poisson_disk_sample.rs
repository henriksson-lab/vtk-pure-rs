//! Poisson disk sampling on mesh surface.
use crate::data::{CellArray, Points, PolyData};

pub fn poisson_disk_sample(mesh: &PolyData, min_distance: f64, seed: u64) -> PolyData {
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.is_empty() {
        return PolyData::new();
    }
    // Compute triangle areas for weighted sampling
    let areas: Vec<f64> = tris
        .iter()
        .map(|&[a, b, c]| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            0.5 * ((u[1] * v[2] - u[2] * v[1]).powi(2)
                + (u[2] * v[0] - u[0] * v[2]).powi(2)
                + (u[0] * v[1] - u[1] * v[0]).powi(2))
            .sqrt()
        })
        .collect();
    let total_area: f64 = areas.iter().sum();
    if total_area < 1e-15 {
        return PolyData::new();
    }
    let max_samples = (total_area / (min_distance * min_distance * 0.866)).ceil() as usize;
    let max_samples = max_samples.min(100000);
    let mut rng = seed;
    let mut next_rand = || -> f64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng >> 33) as f64) / (u32::MAX as f64)
    };
    let mut samples: Vec<[f64; 3]> = Vec::new();
    let min_d2 = min_distance * min_distance;
    for _ in 0..max_samples * 30 {
        // Pick random triangle weighted by area
        let r = next_rand() * total_area;
        let mut acc = 0.0;
        let mut ti = 0;
        for (i, &a) in areas.iter().enumerate() {
            acc += a;
            if acc >= r {
                ti = i;
                break;
            }
        }
        let [a, b, c] = tris[ti];
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let mut u = next_rand();
        let mut v = next_rand();
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;
        let pt = [
            pa[0] * w + pb[0] * u + pc[0] * v,
            pa[1] * w + pb[1] * u + pc[1] * v,
            pa[2] * w + pb[2] * u + pc[2] * v,
        ];
        // Check distance to existing samples
        let too_close = samples.iter().any(|s| {
            (s[0] - pt[0]).powi(2) + (s[1] - pt[1]).powi(2) + (s[2] - pt[2]).powi(2) < min_d2
        });
        if !too_close {
            samples.push(pt);
        }
        if samples.len() >= max_samples {
            break;
        }
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for (i, s) in samples.iter().enumerate() {
        pts.push(*s);
        verts.push_cell(&[i as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.verts = verts;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_poisson() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = poisson_disk_sample(&mesh, 1.0, 42);
        assert!(r.points.len() > 5);
    }
}
