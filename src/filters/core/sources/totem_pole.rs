//! Totem pole (stacked carved cylinders).
use crate::data::{CellArray, Points, PolyData};
pub fn totem_pole(
    height: f64,
    base_radius: f64,
    num_sections: usize,
    resolution: usize,
) -> PolyData {
    let ns = num_sections.max(2);
    let res = resolution.max(6);
    let section_h = height / ns as f64;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut rng = 12345u64;
    for si in 0..ns {
        let z0 = si as f64 * section_h;
        let z1 = z0 + section_h;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let bulge = 0.8 + 0.4 * ((rng >> 33) as f64 / u32::MAX as f64);
        let r0 = base_radius * bulge;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r1 = base_radius * (0.8 + 0.4 * ((rng >> 33) as f64 / u32::MAX as f64));
        let mid_r = base_radius * 1.1;
        // 3 rings per section (bottom, middle bulge, top)
        for (z, r) in [(z0, r0), (z0 + section_h * 0.5, mid_r), (z1, r1)] {
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([r * a.cos(), r * a.sin(), z]);
            }
        }
        let base = pts.len() - res * 3;
        for ring in 0..2 {
            for i in 0..res {
                let j = (i + 1) % res;
                polys.push_cell(&[
                    (base + ring * res + i) as i64,
                    (base + ring * res + j) as i64,
                    (base + (ring + 1) * res + j) as i64,
                    (base + (ring + 1) * res + i) as i64,
                ]);
            }
        }
    }
    // Top cap
    let tc = pts.len();
    pts.push([0.0, 0.0, height + base_radius * 0.3]);
    let top_ring = pts.len() - 1 - res;
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[tc as i64, (top_ring + i) as i64, (top_ring + j) as i64]);
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
        let t = totem_pole(5.0, 0.5, 4, 8);
        assert!(t.polys.num_cells() > 20);
    }
}
