//! Stadium running track (oval shape with lane markings).
use crate::data::{CellArray, Points, PolyData};
pub fn running_track(
    straight_length: f64,
    radius: f64,
    lanes: usize,
    lane_width: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nl = lanes.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for lane in 0..=nl {
        let r = radius + lane as f64 * lane_width;
        let hl = straight_length / 2.0;
        let _n = res * 2 + 2;
        // Right semicircle
        for i in 0..=res {
            let a = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * i as f64 / res as f64;
            pts.push([hl + r * a.cos(), r * a.sin(), 0.0]);
        }
        // Left semicircle
        for i in 0..=res {
            let a = std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * i as f64 / res as f64;
            pts.push([-hl + r * a.cos(), r * a.sin(), 0.0]);
        }
    }
    let ring_size = (res + 1) * 2;
    for lane in 0..nl {
        let r0 = lane * ring_size;
        let r1 = (lane + 1) * ring_size;
        for i in 0..ring_size - 1 {
            polys.push_cell(&[
                (r0 + i) as i64,
                (r0 + i + 1) as i64,
                (r1 + i + 1) as i64,
                (r1 + i) as i64,
            ]);
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
        let t = running_track(50.0, 15.0, 4, 1.2, 16);
        assert!(t.polys.num_cells() > 30);
    }
}
