//! Space debris field (random orbital fragments).
use crate::data::{CellArray, Points, PolyData};
pub fn debris_field(
    num_objects: usize,
    orbit_r_min: f64,
    orbit_r_max: f64,
    max_size: f64,
    seed: u64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut rng = seed;
    let next = |rng: &mut u64| -> f64 {
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*rng >> 33) as f64) / (u32::MAX as f64)
    };
    for _ in 0..num_objects {
        let r = orbit_r_min + (orbit_r_max - orbit_r_min) * next(&mut rng);
        let theta = 2.0 * std::f64::consts::PI * next(&mut rng);
        let phi = (next(&mut rng) - 0.5) * 0.2; // near equatorial
        let x = r * theta.cos() * phi.cos();
        let y = r * theta.sin() * phi.cos();
        let z = r * phi.sin();
        let size = max_size * next(&mut rng) * 0.5 + max_size * 0.1;
        // Random triangle fragment
        let b = pts.len();
        pts.push([
            x + size * next(&mut rng),
            y + size * next(&mut rng),
            z + size * next(&mut rng),
        ]);
        pts.push([
            x + size * next(&mut rng),
            y + size * next(&mut rng),
            z + size * next(&mut rng),
        ]);
        pts.push([
            x + size * next(&mut rng),
            y + size * next(&mut rng),
            z + size * next(&mut rng),
        ]);
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64]);
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
        let d = debris_field(50, 6500.0, 7000.0, 0.1, 42);
        assert_eq!(d.polys.num_cells(), 50);
    }
}
