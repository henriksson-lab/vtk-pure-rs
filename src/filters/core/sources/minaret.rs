//! Minaret (mosque tower) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn minaret(
    base_radius: f64,
    height: f64,
    balcony_height: f64,
    spire_height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main shaft (tapered cylinder)
    let sections = 4;
    let top_r = base_radius * 0.6;
    for is in 0..=sections {
        let t = is as f64 / sections as f64;
        let r = base_radius * (1.0 - t) + top_r * t;
        let z = t * balcony_height;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for is in 0..sections {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (is * res + i) as i64,
                (is * res + j) as i64,
                ((is + 1) * res + j) as i64,
                ((is + 1) * res + i) as i64,
            ]);
        }
    }
    // Balcony (wider ring)
    let balc_r = base_radius * 1.3;
    let bz = balcony_height;
    let bb = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([top_r * a.cos(), top_r * a.sin(), bz]);
        pts.push([balc_r * a.cos(), balc_r * a.sin(), bz]);
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (bb + i * 2) as i64,
            (bb + i * 2 + 1) as i64,
            (bb + j * 2 + 1) as i64,
            (bb + j * 2) as i64,
        ]);
    }
    // Upper shaft
    let us = pts.len();
    let upper_r = top_r * 0.7;
    let upper_h = height - balcony_height - spire_height;
    for ring in 0..=1 {
        let z = balcony_height + if ring == 0 { 0.0 } else { upper_h };
        let r = if ring == 0 { top_r } else { upper_r };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (us + i) as i64,
            (us + j) as i64,
            (us + res + j) as i64,
            (us + res + i) as i64,
        ]);
    }
    // Spire (cone)
    let sc = pts.len();
    pts.push([0.0, 0.0, height]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[sc as i64, (us + res + i) as i64, (us + res + j) as i64]);
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
        let m = minaret(1.5, 15.0, 8.0, 3.0, 12);
        assert!(m.polys.num_cells() > 30);
    }
}
