//! Surface sampling: generate point clouds from mesh surfaces.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Sample points uniformly on a triangle mesh surface.
///
/// Uses random barycentric coordinates for uniform distribution.
pub fn sample_surface_uniform_random(mesh: &PolyData, n_samples: usize, seed: u64) -> PolyData {
    let mut rng = SimpleRng::new(seed);
    let areas = compute_triangle_areas(mesh);
    let total_area: f64 = areas.iter().sum();
    if total_area < 1e-15 || n_samples == 0 {
        return PolyData::new();
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut points = Points::<f64>::new();
    let mut face_id_data = Vec::with_capacity(n_samples);

    // Build CDF for area-proportional sampling
    let mut cdf = Vec::with_capacity(areas.len());
    let mut acc = 0.0;
    for &a in &areas {
        acc += a / total_area;
        cdf.push(acc);
    }

    for _ in 0..n_samples {
        // Pick triangle proportional to area
        let r = rng.next_f64();
        let ci = cdf.partition_point(|&c| c < r).min(cdf.len() - 1);

        let cell = &all_cells[ci];
        if cell.len() < 3 {
            continue;
        }

        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);

        // Random barycentric coordinates
        let u = rng.next_f64();
        let v = rng.next_f64();
        let (s, t) = if u + v > 1.0 {
            (1.0 - u, 1.0 - v)
        } else {
            (u, v)
        };
        let w = 1.0 - s - t;

        let p = [
            w * a[0] + s * b[0] + t * c[0],
            w * a[1] + s * b[1] + t * c[1],
            w * a[2] + s * b[2] + t * c[2],
        ];
        points.push(p);
        face_id_data.push(ci as f64);
    }

    let mut result = PolyData::new();
    result.points = points;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceId",
            face_id_data,
            1,
        )));
    result
}

/// Sample points on a grid pattern over each triangle.
pub fn sample_surface_grid(mesh: &PolyData, subdivisions: usize) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut points = Points::<f64>::new();
    let n = subdivisions.max(1);

    for cell in &all_cells {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);

        for i in 0..=n {
            for j in 0..=n - i {
                let s = i as f64 / n as f64;
                let t = j as f64 / n as f64;
                let w = 1.0 - s - t;
                if w < -1e-10 {
                    continue;
                }
                points.push([
                    w * a[0] + s * b[0] + t * c[0],
                    w * a[1] + s * b[1] + t * c[1],
                    w * a[2] + s * b[2] + t * c[2],
                ]);
            }
        }
    }

    let mut result = PolyData::new();
    result.points = points;
    result
}

fn compute_triangle_areas(mesh: &PolyData) -> Vec<f64> {
    mesh.polys
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return 0.0;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            0.5 * (nx * nx + ny * ny + nz * nz).sqrt()
        })
        .collect()
}

struct SimpleRng {
    state: u64,
}
impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_sampling() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let samples = sample_surface_uniform_random(&mesh, 100, 42);
        assert_eq!(samples.points.len(), 100);
        assert!(samples.point_data().get_array("FaceId").is_some());

        // All points should be in [0,1] x [0,1] x {0}
        for i in 0..samples.points.len() {
            let p = samples.points.get(i);
            assert!(p[0] >= -0.01 && p[0] <= 1.01);
            assert!(p[1] >= -0.01 && p[1] <= 1.01);
            assert!(p[2].abs() < 0.01);
        }
    }

    #[test]
    fn grid_sampling() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let samples = sample_surface_grid(&mesh, 3);
        assert_eq!(samples.points.len(), 10); // (3+1)*(3+2)/2 = 10
    }

    #[test]
    fn empty() {
        let result = sample_surface_uniform_random(&PolyData::new(), 100, 0);
        assert_eq!(result.points.len(), 0);
    }
}
