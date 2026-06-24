//! Historical guillotine frame structure.
use crate::data::{CellArray, Points, PolyData};

pub fn guillotine(height: f64, width: f64) -> PolyData {
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Two uprights
    let l0 = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    let l1 = pts.len();
    pts.push([-hw, 0.0, height]);
    lines.push_cell(&[l0 as i64, l1 as i64]);
    let r0 = pts.len();
    pts.push([hw, 0.0, 0.0]);
    let r1 = pts.len();
    pts.push([hw, 0.0, height]);
    lines.push_cell(&[r0 as i64, r1 as i64]);
    // Top crossbar
    lines.push_cell(&[l1 as i64, r1 as i64]);
    // Blade (triangle at 2/3 height)
    let blade_z = height * 0.67;
    let bb = pts.len();
    pts.push([-hw + 0.05, 0.0, blade_z + width * 0.3]);
    pts.push([hw - 0.05, 0.0, blade_z + width * 0.3]);
    pts.push([hw - 0.05, 0.0, blade_z]);
    pts.push([-hw + 0.05, 0.0, blade_z + 0.02]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // Base platform
    let pb = pts.len();
    let pd = width * 0.3;
    pts.push([-hw - 0.2, -pd, 0.0]);
    pts.push([hw + 0.2, -pd, 0.0]);
    pts.push([hw + 0.2, pd, 0.0]);
    pts.push([-hw - 0.2, pd, 0.0]);
    polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
    // Lunette (semicircle at base of uprights)
    let na = 8;
    let lunette_r = width * 0.12;
    let lunette_z = height * 0.15;
    let lb = pts.len();
    for j in 0..=na {
        let a = std::f64::consts::PI * j as f64 / na as f64;
        pts.push([lunette_r * a.cos(), 0.01, lunette_z + lunette_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(lb + j) as i64, (lb + j + 1) as i64]);
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
    fn test_guillotine() {
        let m = guillotine(4.0, 1.0);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() >= 2);
        assert!(m.lines.num_cells() > 5);
    }
}
