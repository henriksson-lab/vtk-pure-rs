//! Space telescope (Hubble/JWST-like) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn hubble_telescope(
    tube_r: f64,
    tube_l: f64,
    solar_w: f64,
    solar_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Main tube
    for ring in 0..=1 {
        let x = if ring == 0 { 0.0 } else { tube_l };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, tube_r * a.cos(), tube_r * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Aperture door (open)
    let ac = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[ac as i64, j as i64, i as i64]);
    }
    // Solar panels
    for &side in &[-1.0f64, 1.0] {
        let py = side * (tube_r + solar_w / 2.0 + tube_r * 0.3);
        let pb = pts.len();
        pts.push([tube_l * 0.3, -solar_h / 2.0 + py, 0.0]);
        pts.push([tube_l * 0.7, -solar_h / 2.0 + py, 0.0]);
        pts.push([tube_l * 0.7, solar_h / 2.0 + py, 0.0]);
        pts.push([tube_l * 0.3, solar_h / 2.0 + py, 0.0]);
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
        // Panel arm
        let ab = pts.len();
        pts.push([tube_l * 0.5, side * tube_r, 0.0]);
        pts.push([tube_l * 0.5, py, 0.0]);
        lines.push_cell(&[ab as i64, (ab + 1) as i64]);
    }
    // High-gain antenna
    let dish_r = tube_r * 0.5;
    let dish_z = tube_l;
    let dc = pts.len();
    pts.push([dish_z + dish_r * 0.3, 0.0, tube_r * 1.3]);
    for i in 0..res / 2 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
        pts.push([dish_z, dish_r * a.cos(), tube_r * 1.3 + dish_r * a.sin()]);
    }
    for i in 0..res / 2 {
        let j = if i + 1 < res / 2 { dc + 2 + i } else { dc + 1 };
        polys.push_cell(&[dc as i64, (dc + 1 + i) as i64, j as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
pub fn jwst_telescope(mirror_r: f64, sunshield_w: f64, sunshield_l: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Hexagonal mirror segments (18 segments in 3 rings)
    let seg_r = mirror_r / 3.0;
    let hex_h = seg_r * 3.0f64.sqrt() / 2.0;
    let offsets = [
        [0.0, 0.0],
        [seg_r * 1.5, hex_h],
        [seg_r * 1.5, -hex_h],
        [-seg_r * 1.5, hex_h],
        [-seg_r * 1.5, -hex_h],
        [0.0, 2.0 * hex_h],
        [0.0, -2.0 * hex_h],
        [seg_r * 3.0, 0.0],
        [-seg_r * 3.0, 0.0],
        [seg_r * 1.5, 3.0 * hex_h],
        [seg_r * 1.5, -3.0 * hex_h],
        [-seg_r * 1.5, 3.0 * hex_h],
        [-seg_r * 1.5, -3.0 * hex_h],
        [seg_r * 3.0, 2.0 * hex_h],
        [seg_r * 3.0, -2.0 * hex_h],
        [-seg_r * 3.0, 2.0 * hex_h],
        [-seg_r * 3.0, -2.0 * hex_h],
        [0.0, 4.0 * hex_h],
    ];
    for &[ox, oy] in &offsets {
        let b = pts.len();
        for k in 0..6 {
            let a = std::f64::consts::PI / 3.0 * k as f64 + std::f64::consts::PI / 6.0;
            pts.push([0.0, ox + seg_r * a.cos(), oy + seg_r * a.sin()]);
        }
        polys.push_cell(&(0..6).map(|k| (b + k) as i64).collect::<Vec<_>>());
    }
    // Sunshield (5 layers simplified as one quad)
    let sw = sunshield_w / 2.0;
    let sl = sunshield_l / 2.0;
    let sb = pts.len();
    pts.push([-mirror_r * 0.3, -sw, -sl]);
    pts.push([-mirror_r * 0.3, sw, -sl]);
    pts.push([-mirror_r * 0.3, sw, sl]);
    pts.push([-mirror_r * 0.3, -sw, sl]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
    // Secondary mirror support struts
    let sm = pts.len();
    pts.push([mirror_r * 0.5, 0.0, 0.0]); // secondary mirror position
    for i in 0..3 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0;
        let ei = pts.len();
        pts.push([0.0, mirror_r * 0.9 * a.cos(), mirror_r * 0.9 * a.sin()]);
        lines.push_cell(&[sm as i64, ei as i64]);
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
    fn test_hubble() {
        let h = hubble_telescope(1.2, 6.0, 4.0, 1.5, 8);
        assert!(h.polys.num_cells() > 10);
    }
    #[test]
    fn test_jwst() {
        let j = jwst_telescope(3.0, 14.0, 21.0);
        assert!(j.polys.num_cells() >= 18);
    }
}
