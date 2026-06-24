//! Water well geometry (circular wall + roof + bucket mechanism).
use crate::data::{CellArray, Points, PolyData};
pub fn well(well_radius: f64, wall_height: f64, roof_height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Circular wall
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { wall_height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([well_radius * a.cos(), well_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Roof posts
    let pr = well_radius * 1.1;
    for &a in &[0.0f64, std::f64::consts::PI] {
        let b = pts.len();
        pts.push([pr * a.cos(), pr * a.sin(), wall_height]);
        pts.push([pr * a.cos(), pr * a.sin(), wall_height + roof_height]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    // Ridge beam
    let _rb = pts.len() - 2;
    let _rb2 = pts.len() - 1;
    // Roof panels
    let rp = pts.len();
    pts.push([-pr * 1.3, 0.0, wall_height + roof_height * 0.5]);
    pts.push([0.0, 0.0, wall_height + roof_height * 1.1]);
    pts.push([pr * 1.3, 0.0, wall_height + roof_height * 0.5]);
    polys.push_cell(&[rp as i64, (rp + 1) as i64, (rp + 2) as i64]);
    // Crossbar with bucket
    let cb = pts.len();
    pts.push([-pr, 0.0, wall_height + roof_height * 0.7]);
    pts.push([pr, 0.0, wall_height + roof_height * 0.7]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    // Bucket rope
    let bb = pts.len();
    pts.push([0.0, 0.0, wall_height + roof_height * 0.7]);
    pts.push([0.0, 0.0, wall_height * 0.3]);
    lines.push_cell(&[bb as i64, (bb + 1) as i64]);
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
        let w = well(0.8, 1.0, 1.0, 12);
        assert!(w.polys.num_cells() > 10);
        assert!(w.lines.num_cells() > 3);
    }
}
