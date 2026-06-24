//! Blacksmith's anvil.
use crate::data::{CellArray, Points, PolyData};

pub fn anvil(length: f64, height: f64, width: f64) -> PolyData {
    let hl = length / 2.0;
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base (wider)
    let bw = hw * 1.3;
    let bl = hl * 0.6;
    let bb = pts.len();
    pts.push([-bl, -bw, 0.0]);
    pts.push([bl, -bw, 0.0]);
    pts.push([bl, bw, 0.0]);
    pts.push([-bl, bw, 0.0]);
    pts.push([-bl, -bw, height * 0.3]);
    pts.push([bl, -bw, height * 0.3]);
    pts.push([bl, bw, height * 0.3]);
    pts.push([-bl, bw, height * 0.3]);
    for s in 0..4 {
        let i = bb + s;
        let j = bb + (s + 1) % 4;
        let k = bb + 4 + (s + 1) % 4;
        let l = bb + 4 + s;
        polys.push_cell(&[i as i64, j as i64, k as i64, l as i64]);
    }
    polys.push_cell(&[bb as i64, (bb + 3) as i64, (bb + 2) as i64, (bb + 1) as i64]); // bottom
                                                                                      // Waist (narrower)
    let ww = hw * 0.8;
    let wl = hl * 0.5;
    let wb = pts.len();
    pts.push([-wl, -ww, height * 0.3]);
    pts.push([wl, -ww, height * 0.3]);
    pts.push([wl, ww, height * 0.3]);
    pts.push([-wl, ww, height * 0.3]);
    pts.push([-wl, -ww, height * 0.7]);
    pts.push([wl, -ww, height * 0.7]);
    pts.push([wl, ww, height * 0.7]);
    pts.push([-wl, ww, height * 0.7]);
    for s in 0..4 {
        let i = wb + s;
        let j = wb + (s + 1) % 4;
        let k = wb + 4 + (s + 1) % 4;
        let l = wb + 4 + s;
        polys.push_cell(&[i as i64, j as i64, k as i64, l as i64]);
    }
    // Face (top working surface + horn)
    let fb = pts.len();
    pts.push([-hl, -hw, height]);
    pts.push([hl * 0.3, -hw, height]);
    pts.push([hl * 0.3, hw, height]);
    pts.push([-hl, hw, height]);
    polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64, (fb + 3) as i64]); // top
                                                                                      // Horn (triangular extension)
    let horn = pts.len();
    pts.push([hl, 0.0, height * 0.85]);
    polys.push_cell(&[(fb + 1) as i64, horn as i64, (fb + 2) as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_anvil() {
        let m = anvil(4.0, 2.0, 1.5);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() > 8);
    }
}
