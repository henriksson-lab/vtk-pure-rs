//! Heat sink with fins.
use crate::data::{CellArray, Points, PolyData};
pub fn heat_sink(
    base_w: f64,
    base_l: f64,
    base_h: f64,
    fin_h: f64,
    num_fins: usize,
    fin_thickness: f64,
) -> PolyData {
    let nf = num_fins.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let ab = |pts: &mut Points<f64>,
              polys: &mut CellArray,
              x0: f64,
              y0: f64,
              z0: f64,
              x1: f64,
              y1: f64,
              z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    // Base plate
    ab(&mut pts, &mut polys, 0.0, 0.0, 0.0, base_l, base_w, base_h);
    // Fins
    let fin_spacing = base_w / (nf + 1) as f64;
    for fi in 0..nf {
        let y = (fi + 1) as f64 * fin_spacing;
        ab(
            &mut pts,
            &mut polys,
            0.0,
            y - fin_thickness / 2.0,
            base_h,
            base_l,
            y + fin_thickness / 2.0,
            base_h + fin_h,
        );
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
        let h = heat_sink(0.04, 0.04, 0.003, 0.02, 8, 0.001);
        assert_eq!(h.polys.num_cells(), 54);
    } // 1 base + 8 fins = 9 * 6 faces
}
