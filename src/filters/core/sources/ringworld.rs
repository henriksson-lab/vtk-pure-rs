//! Ringworld (Niven ring) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn ringworld(ring_r: f64, ring_w: f64, wall_h: f64, resolution: usize) -> PolyData {
    let res = resolution.max(12);
    let hw = ring_w / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Inner surface (habitable)
    for ring in 0..=1 {
        let z = if ring == 0 { -hw } else { hw };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([ring_r * a.cos(), ring_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Rim walls
    for ring_idx in [0, res] {
        let wall_base = pts.len();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            let wr = ring_r + wall_h;
            pts.push([
                wr * a.cos(),
                wr * a.sin(),
                if ring_idx == 0 { -hw } else { hw },
            ]);
        }
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (ring_idx + i) as i64,
                (ring_idx + j) as i64,
                (wall_base + j) as i64,
                (wall_base + i) as i64,
            ]);
        }
    }
    // Outer surface
    let outer_base = pts.len();
    let or = ring_r + wall_h;
    for ring in 0..=1 {
        let z = if ring == 0 { -hw } else { hw };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([or * a.cos(), or * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (outer_base + j) as i64,
            (outer_base + i) as i64,
            (outer_base + res + i) as i64,
            (outer_base + res + j) as i64,
        ]);
    }
    // Shadow squares (simplified as lines across the ring)
    let mut lines = CellArray::new();
    let sq_count = 4;
    for si in 0..sq_count {
        let a = 2.0 * std::f64::consts::PI * si as f64 / sq_count as f64;
        let inner_r = ring_r * 0.5;
        let sb = pts.len();
        pts.push([inner_r * a.cos(), inner_r * a.sin(), -hw * 0.3]);
        pts.push([
            inner_r * (a + 0.3).cos(),
            inner_r * (a + 0.3).sin(),
            hw * 0.3,
        ]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let r = ringworld(100.0, 10.0, 5.0, 24);
        assert!(r.polys.num_cells() > 50);
    }
}
