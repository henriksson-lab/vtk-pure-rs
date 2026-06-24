//! I-beam and H-beam cross-section extruded geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn i_beam(flange_w: f64, flange_h: f64, web_h: f64, web_t: f64, length: f64) -> PolyData {
    let fw2 = flange_w / 2.0;
    let wt2 = web_t / 2.0;
    let th = flange_h + web_h + flange_h;
    let th2 = th / 2.0;
    let profile = [
        [-fw2, -th2],
        [fw2, -th2],
        [fw2, -th2 + flange_h],
        [wt2, -th2 + flange_h],
        [wt2, th2 - flange_h],
        [fw2, th2 - flange_h],
        [fw2, th2],
        [-fw2, th2],
        [-fw2, th2 - flange_h],
        [-wt2, th2 - flange_h],
        [-wt2, -th2 + flange_h],
        [-fw2, -th2 + flange_h],
    ];
    let np = profile.len();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for p in &profile {
        pts.push([p[0], p[1], 0.0]);
    }
    for p in &profile {
        pts.push([p[0], p[1], length]);
    }
    // Front face (fan)
    for i in 1..np - 1 {
        polys.push_cell(&[0, (i + 1) as i64, i as i64]);
    }
    // Back face
    for i in 1..np - 1 {
        polys.push_cell(&[np as i64, (np + i) as i64, (np + i + 1) as i64]);
    }
    // Sides
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[i as i64, j as i64, (np + j) as i64, (np + i) as i64]);
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
        let b = i_beam(1.0, 0.1, 0.8, 0.1, 5.0);
        assert_eq!(b.points.len(), 24);
        assert!(b.polys.num_cells() > 10);
    }
}
