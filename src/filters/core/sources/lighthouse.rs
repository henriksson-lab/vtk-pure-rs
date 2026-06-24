//! Lighthouse geometry (tapered tower + lamp room + gallery).
use crate::data::{CellArray, Points, PolyData};
pub fn lighthouse(
    base_radius: f64,
    top_radius: f64,
    height: f64,
    lamp_radius: f64,
    lamp_height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Tapered tower
    let tower_sections = 5;
    for is in 0..=tower_sections {
        let t = is as f64 / tower_sections as f64;
        let r = base_radius * (1.0 - t) + top_radius * t;
        let z = t * height;
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for is in 0..tower_sections {
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
    // Gallery (flat ring at top)
    let gallery_r = top_radius * 1.5;
    let gz = height;
    let gb = pts.len();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([top_radius * a.cos(), top_radius * a.sin(), gz]);
        pts.push([gallery_r * a.cos(), gallery_r * a.sin(), gz]);
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (gb + i * 2) as i64,
            (gb + i * 2 + 1) as i64,
            (gb + j * 2 + 1) as i64,
            (gb + j * 2) as i64,
        ]);
    }
    // Lamp room (cylinder on top)
    let lb = pts.len();
    for ring in 0..=1 {
        let z = height + if ring == 0 { 0.0 } else { lamp_height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([lamp_radius * a.cos(), lamp_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (lb + i) as i64,
            (lb + j) as i64,
            (lb + res + j) as i64,
            (lb + res + i) as i64,
        ]);
    }
    // Dome on top
    let dc = pts.len();
    pts.push([0.0, 0.0, height + lamp_height + lamp_radius * 0.5]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[dc as i64, (lb + res + i) as i64, (lb + res + j) as i64]);
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
        let l = lighthouse(3.0, 1.5, 15.0, 1.0, 3.0, 12);
        assert!(l.polys.num_cells() > 30);
    }
}
