//! Uniformly sample random points on mesh surface (area-weighted).
use crate::data::{CellArray, Points, PolyData};

pub fn uniform_sample(mesh: &PolyData, n_samples: usize, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .flat_map(|c| {
            (1..c.len() - 1).filter_map(move |i| {
                if c[0] < 0 || c[i] < 0 || c[i + 1] < 0 {
                    None
                } else {
                    let tri = [c[0] as usize, c[i] as usize, c[i + 1] as usize];
                    if tri.iter().all(|&id| id < n) {
                        Some(tri)
                    } else {
                        None
                    }
                }
            })
        })
        .collect();
    if tris.is_empty() {
        return PolyData::new();
    }
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
    // Cumulative distribution
    let total: f64 = areas.iter().sum();
    if total < 1e-15 {
        return PolyData::new();
    }
    let cdf: Vec<f64> = areas
        .iter()
        .scan(0.0, |acc, &a| {
            *acc += a / total;
            Some(*acc)
        })
        .collect();
    let mut rng = seed;
    let mut next_rand = || -> f64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    };
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for _ in 0..n_samples {
        let r = next_rand();
        let ti = cdf.iter().position(|&c| c >= r).unwrap_or(tris.len() - 1);
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
        let idx = pts.len();
        pts.push([
            pa[0] * w + pb[0] * u + pc[0] * v,
            pa[1] * w + pb[1] * u + pc[1] * v,
            pa[2] * w + pb[2] * u + pc[2] * v,
        ]);
        verts.push_cell(&[idx as i64]);
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
    fn test_uniform() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = uniform_sample(&mesh, 50, 42);
        assert_eq!(r.points.len(), 50);
    }

    #[test]
    fn skips_invalid_cells_before_sampling() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 99]);
        mesh.polys.push_cell(&[0, -1, 2]);
        mesh.polys.push_cell(&[0, 1, 2]);

        let r = uniform_sample(&mesh, 16, 7);
        assert_eq!(r.points.len(), 16);
    }
}
