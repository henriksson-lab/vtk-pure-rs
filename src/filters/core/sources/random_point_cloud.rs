//! Random point cloud generators.

use crate::data::{CellArray, Points, PolyData};

/// Generate random points in a box.
pub fn random_points_box(n: usize, bounds: [[f64; 2]; 3], seed: u64) -> PolyData {
    let mut rng = Rng(seed);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..n {
        pts.push([
            bounds[0][0] + rng.next() * (bounds[0][1] - bounds[0][0]),
            bounds[1][0] + rng.next() * (bounds[1][1] - bounds[1][0]),
            bounds[2][0] + rng.next() * (bounds[2][1] - bounds[2][0]),
        ]);
        verts.push_cell(&[i as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

/// Generate random points on a sphere surface.
pub fn random_points_sphere(n: usize, radius: f64, seed: u64) -> PolyData {
    let mut rng = Rng(seed);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..n {
        let z = 2.0 * rng.next() - 1.0;
        let phi = 2.0 * std::f64::consts::PI * rng.next();
        let rxy = (1.0 - z * z).sqrt();
        pts.push([
            radius * rxy * phi.cos(),
            radius * rxy * phi.sin(),
            radius * z,
        ]);
        verts.push_cell(&[i as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

/// Generate Gaussian distributed points.
pub fn random_points_gaussian(n: usize, center: [f64; 3], sigma: [f64; 3], seed: u64) -> PolyData {
    let mut rng = Rng(seed);
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..n {
        pts.push([
            center[0] + sigma[0] * box_muller(&mut rng),
            center[1] + sigma[1] * box_muller(&mut rng),
            center[2] + sigma[2] * box_muller(&mut rng),
        ]);
        verts.push_cell(&[i as i64]);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

fn box_muller(rng: &mut Rng) -> f64 {
    let u1 = rng.next().max(1e-30);
    let u2 = rng.next();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 32) as u32 as f64) / (u32::MAX as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_box() {
        let pc = random_points_box(100, [[0.0, 1.0], [0.0, 1.0], [0.0, 1.0]], 42);
        assert_eq!(pc.points.len(), 100);
        for i in 0..100 {
            let p = pc.points.get(i);
            assert!(p[0] >= 0.0 && p[0] <= 1.0);
        }
    }
    #[test]
    fn test_sphere() {
        let pc = random_points_sphere(50, 1.0, 42);
        assert_eq!(pc.points.len(), 50);
        for i in 0..50 {
            let p = pc.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-10);
        }
    }
    #[test]
    fn test_gaussian() {
        let pc = random_points_gaussian(200, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 42);
        assert_eq!(pc.points.len(), 200);
    }
}
