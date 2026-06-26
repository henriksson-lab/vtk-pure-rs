//! Stratified sampling on mesh surfaces: blue noise, jittered grid, importance.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Blue-noise-like sampling using dart throwing on mesh surface.
pub fn dart_throwing_sample(
    mesh: &PolyData,
    min_distance: f64,
    max_samples: usize,
    seed: u64,
) -> PolyData {
    let tris = surface_tris(mesh);
    let total: f64 = tris.iter().map(|t| t.area).sum();
    if total < 1e-15 {
        return PolyData::new();
    }

    let mut rng = seed.wrapping_add(1);
    let next = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s >> 33) as f64 / (1u64 << 31) as f64
    };

    let mut cdf = Vec::with_capacity(tris.len());
    let mut acc = 0.0;
    for tri in &tris {
        acc += tri.area / total;
        cdf.push(acc);
    }

    let mut accepted: Vec<[f64; 3]> = Vec::new();
    let d2 = min_distance * min_distance;

    for _ in 0..max_samples * 10 {
        if accepted.len() >= max_samples {
            break;
        }
        let r = next(&mut rng);
        let ci = cdf.partition_point(|&c| c < r).min(cdf.len() - 1);
        let tri = tris[ci].ids;

        let u = next(&mut rng);
        let v = next(&mut rng);
        let (s, t) = if u + v > 1.0 {
            (1.0 - u, 1.0 - v)
        } else {
            (u, v)
        };
        let w = 1.0 - s - t;
        let a = mesh.points.get(tri[0]);
        let b = mesh.points.get(tri[1]);
        let c = mesh.points.get(tri[2]);
        let p = [
            w * a[0] + s * b[0] + t * c[0],
            w * a[1] + s * b[1] + t * c[1],
            w * a[2] + s * b[2] + t * c[2],
        ];

        let too_close = accepted
            .iter()
            .any(|q| (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) < d2);
        if !too_close {
            accepted.push(p);
        }
    }

    let mut pts = Points::<f64>::new();
    for p in &accepted {
        pts.push(*p);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
}

/// Importance sampling: more samples where a scalar is higher.
pub fn importance_sample(
    mesh: &PolyData,
    array_name: &str,
    n_samples: usize,
    seed: u64,
) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= mesh.points.len() => a,
        _ => return PolyData::new(),
    };
    let tris = surface_tris(mesh);
    let mut buf = [0.0f64];

    // Weight each triangle by area × average scalar
    let weights: Vec<f64> = tris
        .iter()
        .map(|tri| {
            let avg: f64 = tri
                .ids
                .iter()
                .map(|&pid| {
                    arr.tuple_as_f64(pid, &mut buf);
                    buf[0].max(0.0)
                })
                .sum::<f64>()
                / 3.0;
            tri.area * avg
        })
        .collect();
    let total: f64 = weights.iter().sum();
    if total < 1e-15 {
        return PolyData::new();
    }

    let mut cdf = Vec::new();
    let mut acc = 0.0;
    for &w in &weights {
        acc += w / total;
        cdf.push(acc);
    }

    let mut rng = seed.wrapping_add(1);
    let next = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s >> 33) as f64 / (1u64 << 31) as f64
    };

    let mut pts = Points::<f64>::new();
    let mut val_data = Vec::new();
    for _ in 0..n_samples {
        let r = next(&mut rng);
        let ci = cdf.partition_point(|&c| c < r).min(cdf.len() - 1);
        let tri = tris[ci].ids;
        let u = next(&mut rng);
        let v = next(&mut rng);
        let (s, t) = if u + v > 1.0 {
            (1.0 - u, 1.0 - v)
        } else {
            (u, v)
        };
        let w = 1.0 - s - t;
        let a = mesh.points.get(tri[0]);
        let b = mesh.points.get(tri[1]);
        let c = mesh.points.get(tri[2]);
        pts.push([
            w * a[0] + s * b[0] + t * c[0],
            w * a[1] + s * b[1] + t * c[1],
            w * a[2] + s * b[2] + t * c[2],
        ]);

        arr.tuple_as_f64(tri[0], &mut buf);
        let va = buf[0];
        arr.tuple_as_f64(tri[1], &mut buf);
        let vb = buf[0];
        arr.tuple_as_f64(tri[2], &mut buf);
        let vc = buf[0];
        val_data.push(w * va + s * vb + t * vc);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, val_data, 1,
        )));
    result
}

#[derive(Clone, Copy)]
struct SurfaceTri {
    ids: [usize; 3],
    area: f64,
}

fn surface_tris(mesh: &PolyData) -> Vec<SurfaceTri> {
    let n = mesh.points.len();
    let mut tris = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || cell.iter().any(|&id| id < 0 || id as usize >= n) {
            continue;
        }
        let i0 = cell[0] as usize;
        for i in 1..cell.len() - 1 {
            let ids = [i0, cell[i] as usize, cell[i + 1] as usize];
            let area = tri_area(
                mesh.points.get(ids[0]),
                mesh.points.get(ids[1]),
                mesh.points.get(ids[2]),
            );
            if area > 0.0 {
                tris.push(SurfaceTri { ids, area });
            }
        }
    }
    tris
}

fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
    .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dart_throw() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [10.0, 10.0, 0.0],
                [0.0, 10.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let result = dart_throwing_sample(&mesh, 1.0, 50, 42);
        assert!(result.points.len() > 5);
    }
    #[test]
    fn importance() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [2.5, 5.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![1.0, 1.0, 1.0],
                1,
            )));
        let result = importance_sample(&mesh, "w", 20, 42);
        assert!(result.points.len() >= 10);
    }
}
