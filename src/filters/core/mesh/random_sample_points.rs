//! Random point sampling on mesh surfaces.

use crate::data::{CellArray, Points, PolyData};

/// Uniformly sample N random points on the mesh surface.
pub fn sample_surface_points(mesh: &PolyData, n: usize, seed: u64) -> PolyData {
    let mut triangles = Vec::new();
    let mut areas = Vec::new();

    for cell in mesh.polys.iter().filter(|c| c.len() >= 3) {
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            triangles.push([cell[0], cell[i], cell[i + 1]]);
            areas.push(0.5 * (cx * cx + cy * cy + cz * cz).sqrt());
        }
    }

    if triangles.is_empty() {
        return PolyData::new();
    }

    let total_area: f64 = areas.iter().sum();
    if total_area < 1e-30 {
        return PolyData::new();
    }

    let mut cum_area = Vec::with_capacity(areas.len());
    let mut acc = 0.0;
    for &a in &areas {
        acc += a;
        cum_area.push(acc / total_area);
    }

    let mut rng = SimpleRng(seed);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();

    for _ in 0..n {
        // Pick triangle weighted by area.
        let r = rng.next_f64();
        let ci = cum_area
            .partition_point(|&ca| ca < r)
            .min(triangles.len() - 1);
        let tri = triangles[ci];

        // Random barycentric coordinates
        let mut u = rng.next_f64();
        let mut v = rng.next_f64();
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;

        let a = mesh.points.get(tri[0] as usize);
        let b = mesh.points.get(tri[1] as usize);
        let c = mesh.points.get(tri[2] as usize);
        let idx = pts.len();
        pts.push([
            a[0] * w + b[0] * u + c[0] * v,
            a[1] * w + b[1] * u + c[1] * v,
            a[2] * w + b[2] * u + c[2] * v,
        ]);
        verts.push_cell(&[idx as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

struct SimpleRng(u64);
impl SimpleRng {
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f64) / ((1u64 << 31) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sample() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let pts = sample_surface_points(&mesh, 100, 42);
        assert_eq!(pts.points.len(), 100);
        // All points should be within triangle bounds
        for i in 0..100 {
            let p = pts.points.get(i);
            assert!(p[0] >= -0.1 && p[0] <= 10.1);
            assert!(p[1] >= -0.1 && p[1] <= 10.1);
        }
    }
    #[test]
    fn test_empty() {
        let mesh = PolyData::new();
        let pts = sample_surface_points(&mesh, 10, 42);
        assert_eq!(pts.points.len(), 0);
    }

    #[test]
    fn samples_all_triangles_of_polygon() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let pts = sample_surface_points(&mesh, 200, 7);
        let found_upper_left = (0..pts.points.len()).any(|i| {
            let p = pts.points.get(i);
            p[0] < 0.5 && p[1] > 0.5
        });
        assert!(found_upper_left);
    }
}
