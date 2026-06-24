//! Circular labyrinth geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn circular_labyrinth(rings: usize, radius: f64, wall_height: f64, seed: u64) -> PolyData {
    let nr = rings.max(2);
    let segments = 8;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let _wt = radius * 0.02;
    let mut rng = seed;
    for r in 1..=nr {
        let ring_r = radius * r as f64 / nr as f64;
        for s in 0..segments {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            if (rng >> 33) % 3 == 0 {
                continue;
            } // random gaps
            let a0 = 2.0 * std::f64::consts::PI * s as f64 / segments as f64;
            let a1 = 2.0 * std::f64::consts::PI * (s + 1) as f64 / segments as f64;
            let b = pts.len();
            pts.push([ring_r * a0.cos(), ring_r * a0.sin(), 0.0]);
            pts.push([ring_r * a1.cos(), ring_r * a1.sin(), 0.0]);
            pts.push([ring_r * a1.cos(), ring_r * a1.sin(), wall_height]);
            pts.push([ring_r * a0.cos(), ring_r * a0.sin(), wall_height]);
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let l = circular_labyrinth(4, 5.0, 1.0, 42);
        assert!(l.polys.num_cells() > 5);
    }
}
