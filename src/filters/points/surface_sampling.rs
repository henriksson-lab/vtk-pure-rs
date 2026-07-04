//! Surface sampling: generate point clouds from mesh surfaces.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Sample points uniformly on a triangle mesh surface.
///
/// Uses random barycentric coordinates for uniform distribution.
pub fn sample_surface_uniform_random(mesh: &PolyData, n_samples: usize, seed: u64) -> PolyData {
    let mut rng = SimpleRng::new(seed);
    let triangles = collect_fan_triangles(mesh);
    let total_area: f64 = triangles.iter().map(|t| t.area).sum();
    if total_area < 1e-15 || n_samples == 0 {
        return PolyData::new();
    }

    let mut points = Points::<f64>::new();
    let mut face_id_data = Vec::with_capacity(n_samples);

    // Build CDF for area-proportional sampling
    let mut cdf = Vec::with_capacity(triangles.len());
    let mut acc = 0.0;
    for triangle in &triangles {
        acc += triangle.area / total_area;
        cdf.push(acc);
    }

    for _ in 0..n_samples {
        // Pick triangle proportional to area
        let r = rng.next_f64();
        let ci = cdf.partition_point(|&c| c < r).min(cdf.len() - 1);
        let triangle = &triangles[ci];

        let a = mesh.points.get(triangle.ids[0]);
        let b = mesh.points.get(triangle.ids[1]);
        let c = mesh.points.get(triangle.ids[2]);

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
        face_id_data.push(triangle.cell_id as f64);
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
    let triangles = collect_fan_triangles(mesh);
    let mut points = Points::<f64>::new();
    let n = subdivisions.max(1);

    for triangle in &triangles {
        let a = mesh.points.get(triangle.ids[0]);
        let b = mesh.points.get(triangle.ids[1]);
        let c = mesh.points.get(triangle.ids[2]);

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

#[derive(Clone, Debug)]
struct SurfaceTriangle {
    cell_id: usize,
    ids: [usize; 3],
    area: f64,
}

fn collect_fan_triangles(mesh: &PolyData) -> Vec<SurfaceTriangle> {
    let mut triangles = Vec::new();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        let ids: Vec<usize> = cell.iter().map(|&id| id as usize).collect();
        for i in 1..ids.len() - 1 {
            let tri_ids = [ids[0], ids[i], ids[i + 1]];
            triangles.push(SurfaceTriangle {
                cell_id,
                ids: tri_ids,
                area: triangle_area(mesh, tri_ids),
            });
        }
    }
    triangles
}

fn triangle_area(mesh: &PolyData, ids: [usize; 3]) -> f64 {
    let a = mesh.points.get(ids[0]);
    let b = mesh.points.get(ids[1]);
    let c = mesh.points.get(ids[2]);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (nx * nx + ny * ny + nz * nz).sqrt()
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
    fn grid_sampling_fan_triangulates_quads() {
        let mesh = PolyData::from_quads(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        let samples = sample_surface_grid(&mesh, 1);
        assert_eq!(samples.points.len(), 6);
    }

    #[test]
    fn empty() {
        let result = sample_surface_uniform_random(&PolyData::new(), 100, 0);
        assert_eq!(result.points.len(), 0);
    }
}
