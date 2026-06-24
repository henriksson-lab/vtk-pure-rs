//! O'Neill cylinder space habitat (rotating cylinder with endcaps).
use crate::data::{CellArray, Points, PolyData};
pub fn oneill_cylinder(
    radius: f64,
    length: f64,
    window_strips: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(12);
    let hl = length / 2.0;
    let nw = window_strips.max(0);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main cylinder hull
    for ring in 0..=1 {
        let x = if ring == 0 { -hl } else { hl };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, radius * a.cos(), radius * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        // Skip window strips
        let is_window = nw > 0 && (i % (res / nw.max(1))) == 0;
        if !is_window {
            polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
        }
    }
    // Endcaps (hemispherical)
    for &side in &[-1.0f64, 1.0] {
        let x = side * hl;
        let cap_d = radius * 0.3;
        let _dome_res = res / 2;
        let dc = pts.len();
        pts.push([x + side * cap_d, 0.0, 0.0]);
        let db = pts.len();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, radius * 0.95 * a.cos(), radius * 0.95 * a.sin()]);
        }
        let _ring_base = if side < 0.0 { 0 } else { res };
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[dc as i64, (db + j) as i64, (db + i) as i64]);
        }
    }
    // Interior terrain strips (land areas along inside)
    let land_r = radius * 0.98;
    let land_w = length * 0.8;
    for strip in 0..3 {
        let a0 = 2.0 * std::f64::consts::PI * strip as f64 / 3.0;
        let a1 = a0 + 2.0 * std::f64::consts::PI / 3.0 * 0.6; // 60% land, 40% window
        let sb = pts.len();
        let steps = 8;
        for si in 0..=steps {
            let a = a0 + (a1 - a0) * si as f64 / steps as f64;
            pts.push([-land_w / 2.0, land_r * a.cos(), land_r * a.sin()]);
            pts.push([land_w / 2.0, land_r * a.cos(), land_r * a.sin()]);
        }
        for si in 0..steps {
            let b = sb + si * 2;
            polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
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
        let h = oneill_cylinder(500.0, 2000.0, 3, 24);
        assert!(h.polys.num_cells() > 30);
    }
}
