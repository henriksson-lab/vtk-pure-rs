//! Add random positional noise to mesh vertices.
use crate::data::{Points, PolyData};

pub fn add_noise(mesh: &PolyData, amplitude: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut rng = seed;
    let mut next_rand = || -> f64 {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((rng >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0
    };
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + amplitude * next_rand(),
            p[1] + amplitude * next_rand(),
            p[2] + amplitude * next_rand(),
        ]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_noise() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = add_noise(&mesh, 0.01, 42);
        assert_eq!(r.points.len(), 3);
        // Points should be slightly different
        let p = r.points.get(0);
        assert!((p[0]).abs() < 0.02);
    }
}
