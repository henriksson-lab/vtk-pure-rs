//! Theremin electronic instrument (box with two antennas).
use crate::data::{CellArray, Points, PolyData};

pub fn theremin(box_width: f64, box_height: f64, box_depth: f64, antenna_height: f64) -> PolyData {
    let hw = box_width / 2.0;
    let _hh = box_height / 2.0;
    let hd = box_depth / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Main box
    let bb = pts.len();
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    pts.push([-hw, -hd, box_height]);
    pts.push([hw, -hd, box_height]);
    pts.push([hw, hd, box_height]);
    pts.push([-hw, hd, box_height]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 5) as i64, (bb + 4) as i64]);
    polys.push_cell(&[
        (bb + 1) as i64,
        (bb + 2) as i64,
        (bb + 6) as i64,
        (bb + 5) as i64,
    ]);
    polys.push_cell(&[
        (bb + 2) as i64,
        (bb + 3) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
    ]);
    polys.push_cell(&[(bb + 3) as i64, bb as i64, (bb + 4) as i64, (bb + 7) as i64]);
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 7) as i64,
    ]); // top
        // Pitch antenna (vertical rod on right)
    let pa0 = pts.len();
    pts.push([hw, 0.0, box_height]);
    let pa1 = pts.len();
    pts.push([hw, 0.0, box_height + antenna_height]);
    lines.push_cell(&[pa0 as i64, pa1 as i64]);
    // Volume antenna (horizontal loop on left)
    let va0 = pts.len();
    pts.push([-hw, 0.0, box_height * 0.7]);
    let va1 = pts.len();
    pts.push([-hw - antenna_height * 0.5, 0.0, box_height * 0.7]);
    let va2 = pts.len();
    pts.push([
        -hw - antenna_height * 0.5,
        0.0,
        box_height * 0.7 + antenna_height * 0.3,
    ]);
    let va3 = pts.len();
    pts.push([-hw, 0.0, box_height * 0.7 + antenna_height * 0.3]);
    lines.push_cell(&[va0 as i64, va1 as i64]);
    lines.push_cell(&[va1 as i64, va2 as i64]);
    lines.push_cell(&[va2 as i64, va3 as i64]);
    // Speaker grille (circle on front)
    let spk_r = box_height * 0.2;
    let spk_z = box_height * 0.5;
    let na = 12;
    let sb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([spk_r * a.cos(), -hd - 0.01, spk_z + spk_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(sb + j) as i64, (sb + (j + 1) % na) as i64]);
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
    fn test_theremin() {
        let m = theremin(3.0, 1.5, 1.0, 2.0);
        assert!(m.points.len() > 20);
        assert!(m.polys.num_cells() >= 5);
        assert!(m.lines.num_cells() > 5);
    }
}
