//! Dam geometry (concrete gravity dam with spillway).
use crate::data::{CellArray, Points, PolyData};
pub fn gravity_dam(
    width: f64,
    height: f64,
    base_thickness: f64,
    top_thickness: f64,
    spillway_w: f64,
) -> PolyData {
    let hw = width / 2.0;
    let bt = base_thickness;
    let tt = top_thickness;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Dam cross-section (trapezoidal, extruded along width)
    // Upstream face (vertical)
    pts.push([-hw, 0.0, 0.0]);
    pts.push([hw, 0.0, 0.0]); //0,1
    pts.push([hw, 0.0, height]);
    pts.push([-hw, 0.0, height]); //2,3
    polys.push_cell(&[0, 1, 2, 3]); // upstream face
                                    // Downstream face (sloped)
    pts.push([-hw, bt, 0.0]);
    pts.push([hw, bt, 0.0]); //4,5
    pts.push([hw, tt, height]);
    pts.push([-hw, tt, height]); //6,7
    polys.push_cell(&[5, 4, 7, 6]); // downstream face
                                    // Top
    polys.push_cell(&[3, 2, 6, 7]);
    // Bottom
    polys.push_cell(&[0, 4, 5, 1]);
    // Left side
    polys.push_cell(&[0, 3, 7, 4]);
    // Right side
    polys.push_cell(&[1, 5, 6, 2]);
    // Spillway notch (cut into top center)
    let sw = spillway_w / 2.0;
    let sh = height * 0.1;
    let sb = pts.len();
    pts.push([-sw, 0.0, height - sh]);
    pts.push([sw, 0.0, height - sh]); //8,9
    pts.push([sw, tt, height - sh]);
    pts.push([-sw, tt, height - sh]); //10,11
    polys.push_cell(&[
        (sb) as i64,
        (sb + 1) as i64,
        (sb + 2) as i64,
        (sb + 3) as i64,
    ]); // spillway floor
        // Water surface (upstream, at spillway level)
    let wb = pts.len();
    pts.push([-hw, -height * 2.0, height - sh]);
    pts.push([hw, -height * 2.0, height - sh]);
    pts.push([hw, 0.0, height - sh]);
    pts.push([-hw, 0.0, height - sh]);
    polys.push_cell(&[wb as i64, (wb + 1) as i64, (wb + 2) as i64, (wb + 3) as i64]);
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
        let d = gravity_dam(50.0, 30.0, 20.0, 5.0, 10.0);
        assert!(d.polys.num_cells() >= 7);
    }
}
