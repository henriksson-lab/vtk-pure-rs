//! Road crossroads/intersection geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn crossroads(road_width: f64, road_length: f64) -> PolyData {
    let hw = road_width / 2.0;
    let hl = road_length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // North-south road
    let nb = pts.len();
    pts.push([-hw, -hl, 0.0]);
    pts.push([hw, -hl, 0.0]);
    pts.push([hw, hl, 0.0]);
    pts.push([-hw, hl, 0.0]);
    polys.push_cell(&[nb as i64, (nb + 1) as i64, (nb + 2) as i64, (nb + 3) as i64]);
    // East-west road
    let eb = pts.len();
    pts.push([-hl, -hw, 0.0]);
    pts.push([hl, -hw, 0.0]);
    pts.push([hl, hw, 0.0]);
    pts.push([-hl, hw, 0.0]);
    polys.push_cell(&[eb as i64, (eb + 1) as i64, (eb + 2) as i64, (eb + 3) as i64]);
    // Sidewalk corners (raised)
    let sh = 0.15;
    let sw = road_width * 0.3;
    for &(sx, sy) in &[
        (hw, hw),
        (hw, -hw - sw),
        (-hw - sw, hw),
        (-hw - sw, -hw - sw),
    ] {
        let cb = pts.len();
        pts.push([sx, sy, 0.0]);
        pts.push([sx + sw, sy, 0.0]);
        pts.push([sx + sw, sy + sw, 0.0]);
        pts.push([sx, sy + sw, 0.0]);
        pts.push([sx, sy, sh]);
        pts.push([sx + sw, sy, sh]);
        pts.push([sx + sw, sy + sw, sh]);
        pts.push([sx, sy + sw, sh]);
        let f = |i: usize| (cb + i) as i64;
        polys.push_cell(&[f(4), f(5), f(6), f(7)]); // top
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
        let c = crossroads(4.0, 20.0);
        assert!(c.polys.num_cells() >= 6);
    }
}
