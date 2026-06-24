//! Portable pocket sundial (folding type with compass).
use crate::data::{CellArray, Points, PolyData};

pub fn pocket_sundial(size: f64) -> PolyData {
    let hs = size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Base plate (rectangle)
    let bb = pts.len();
    pts.push([-hs, -hs, 0.0]);
    pts.push([hs, -hs, 0.0]);
    pts.push([hs, hs, 0.0]);
    pts.push([-hs, hs, 0.0]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // Lid (tilted rectangle, hinged at back)
    let lid_angle = 50.0f64 * std::f64::consts::PI / 180.0;
    let lb = pts.len();
    pts.push([-hs, hs, 0.0]);
    pts.push([hs, hs, 0.0]);
    pts.push([hs, hs + size * lid_angle.cos(), size * lid_angle.sin()]);
    pts.push([-hs, hs + size * lid_angle.cos(), size * lid_angle.sin()]);
    polys.push_cell(&[lb as i64, (lb + 1) as i64, (lb + 2) as i64, (lb + 3) as i64]);
    // Compass circle on base
    let comp_r = size * 0.25;
    let na = 16;
    let cb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([comp_r * a.cos(), -hs * 0.3 + comp_r * a.sin(), 0.01]);
    }
    for j in 0..na {
        lines.push_cell(&[(cb + j) as i64, (cb + (j + 1) % na) as i64]);
    }
    // Compass needle
    let cn0 = pts.len();
    pts.push([0.0, -hs * 0.3 - comp_r * 0.8, 0.02]);
    let cn1 = pts.len();
    pts.push([0.0, -hs * 0.3 + comp_r * 0.8, 0.02]);
    lines.push_cell(&[cn0 as i64, cn1 as i64]);
    // Gnomon string (from base to lid)
    let g0 = pts.len();
    pts.push([0.0, hs * 0.3, 0.01]);
    let g1 = pts.len();
    pts.push([
        0.0,
        hs + size * 0.5 * lid_angle.cos(),
        size * 0.5 * lid_angle.sin(),
    ]);
    lines.push_cell(&[g0 as i64, g1 as i64]);
    // Hour lines on base
    for h in 0..7 {
        let angle = -std::f64::consts::PI / 3.0 + std::f64::consts::PI * 2.0 / 3.0 * h as f64 / 6.0;
        let hl0 = pts.len();
        pts.push([0.0, hs * 0.3, 0.005]);
        let hl1 = pts.len();
        pts.push([
            hs * 0.6 * angle.sin(),
            hs * 0.3 + hs * 0.6 * angle.cos(),
            0.005,
        ]);
        lines.push_cell(&[hl0 as i64, hl1 as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pocket_sundial() {
        let m = pocket_sundial(3.0);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() >= 2);
        assert!(m.lines.num_cells() > 10);
    }
}
