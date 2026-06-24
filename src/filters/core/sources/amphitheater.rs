//! Amphitheater (semicircular tiered seating) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn amphitheater(
    radius: f64,
    tiers: usize,
    tier_height: f64,
    tier_depth: f64,
    angle_degrees: f64,
    resolution: usize,
) -> PolyData {
    let nt = tiers.max(1);
    let res = resolution.max(8);
    let angle = angle_degrees.to_radians();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for ti in 0..nt {
        let r_inner = radius + ti as f64 * tier_depth;
        let r_outer = r_inner + tier_depth;
        let z_bottom = ti as f64 * tier_height;
        let z_top = z_bottom + tier_height;
        for i in 0..=res {
            let a = -angle / 2.0 + angle * i as f64 / res as f64;
            let b = pts.len();
            pts.push([r_inner * a.cos(), r_inner * a.sin(), z_bottom]); //0 inner bottom
            pts.push([r_outer * a.cos(), r_outer * a.sin(), z_bottom]); //1 outer bottom
            pts.push([r_outer * a.cos(), r_outer * a.sin(), z_top]); //2 outer top
            pts.push([r_inner * a.cos(), r_inner * a.sin(), z_top]); //3 inner top
            if i > 0 {
                let p = b - 4;
                // Top surface (seat)
                polys.push_cell(&[
                    (p + 3) as i64,
                    (p + 2) as i64,
                    (b + 2) as i64,
                    (b + 3) as i64,
                ]);
                // Front face (riser)
                polys.push_cell(&[
                    (p + 0) as i64,
                    (b + 0) as i64,
                    (b + 3) as i64,
                    (p + 3) as i64,
                ]);
            }
        }
    }
    // Stage floor
    let sc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..=res {
        let a = -angle / 2.0 + angle * i as f64 / res as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for i in 0..res {
        polys.push_cell(&[sc as i64, (sc + 1 + i) as i64, (sc + 2 + i) as i64]);
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
        let a = amphitheater(5.0, 4, 0.5, 1.0, 180.0, 12);
        assert!(a.polys.num_cells() > 30);
    }
}
