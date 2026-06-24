//! Signpost (pole + sign board).
use crate::data::{CellArray, Points, PolyData};
pub fn signpost(pole_height: f64, sign_width: f64, sign_height: f64, pole_radius: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pole (line)
    let pb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, pole_height]);
    lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    // Sign board (quad)
    let sw = sign_width / 2.0;
    let sh = sign_height / 2.0;
    let sz = pole_height - sh;
    let sb = pts.len();
    pts.push([-sw, pole_radius * 1.5, sz - sh]);
    pts.push([sw, pole_radius * 1.5, sz - sh]);
    pts.push([sw, pole_radius * 1.5, sz + sh]);
    pts.push([-sw, pole_radius * 1.5, sz + sh]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
    // Back of sign
    let bb = pts.len();
    pts.push([-sw, -pole_radius * 0.5, sz - sh]);
    pts.push([sw, -pole_radius * 0.5, sz - sh]);
    pts.push([sw, -pole_radius * 0.5, sz + sh]);
    pts.push([-sw, -pole_radius * 0.5, sz + sh]);
    polys.push_cell(&[(bb + 3) as i64, (bb + 2) as i64, (bb + 1) as i64, bb as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
pub fn directional_sign(
    pole_height: f64,
    num_signs: usize,
    sign_width: f64,
    sign_height: f64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let pb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, pole_height]);
    lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    let ns = num_signs.max(1);
    for si in 0..ns {
        let z = pole_height - sign_height * (si as f64 + 0.5);
        let angle = si as f64 * 0.3;
        let sw = sign_width / 2.0;
        let sh = sign_height * 0.4;
        let ca = angle.cos();
        let sa = angle.sin();
        let sb = pts.len();
        pts.push([-sw * ca, sw * sa, z - sh]);
        pts.push([sw * ca, -sw * sa, z - sh]);
        pts.push([sw * ca, -sw * sa, z + sh]);
        pts.push([-sw * ca, sw * sa, z + sh]);
        polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
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
    fn test_single() {
        let s = signpost(3.0, 1.0, 0.5, 0.05);
        assert!(s.polys.num_cells() >= 2);
    }
    #[test]
    fn test_multi() {
        let s = directional_sign(3.0, 3, 1.2, 0.4);
        assert_eq!(s.polys.num_cells(), 3);
    }
}
