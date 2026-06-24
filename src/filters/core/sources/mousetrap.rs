//! Classic snap mousetrap.
use crate::data::{CellArray, Points, PolyData};

pub fn mousetrap(size: f64) -> PolyData {
    let hs = size / 2.0;
    let hd = size * 0.3;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Base board
    let bb = pts.len();
    pts.push([-hs, -hd, 0.0]);
    pts.push([hs, -hd, 0.0]);
    pts.push([hs, hd, 0.0]);
    pts.push([-hs, hd, 0.0]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    // Spring bar (U-shape wire)
    let spring_r = size * 0.35;
    let na = 12;
    let sb = pts.len();
    for j in 0..=na {
        let a = std::f64::consts::PI * j as f64 / na as f64;
        pts.push([spring_r * a.cos(), 0.0, spring_r * a.sin() + 0.02]);
    }
    for j in 0..na {
        lines.push_cell(&[(sb + j) as i64, (sb + j + 1) as i64]);
    }
    // Kill bar (straight wire across)
    let kb0 = pts.len();
    pts.push([-hs * 0.8, -hd * 0.8, 0.01]);
    let kb1 = pts.len();
    pts.push([-hs * 0.8, hd * 0.8, 0.01]);
    lines.push_cell(&[kb0 as i64, kb1 as i64]);
    // Trigger mechanism
    let t0 = pts.len();
    pts.push([hs * 0.3, 0.0, 0.02]);
    let t1 = pts.len();
    pts.push([hs * 0.3, 0.0, size * 0.15]);
    let t2 = pts.len();
    pts.push([hs * 0.1, 0.0, size * 0.15]);
    lines.push_cell(&[t0 as i64, t1 as i64]);
    lines.push_cell(&[t1 as i64, t2 as i64]);
    // Bait pedal
    let bp = pts.len();
    pts.push([hs * 0.5, -hd * 0.3, 0.01]);
    pts.push([hs * 0.8, -hd * 0.3, 0.01]);
    pts.push([hs * 0.8, hd * 0.3, 0.01]);
    pts.push([hs * 0.5, hd * 0.3, 0.01]);
    polys.push_cell(&[bp as i64, (bp + 1) as i64, (bp + 2) as i64, (bp + 3) as i64]);
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
    fn test_mousetrap() {
        let m = mousetrap(3.0);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() >= 2);
        assert!(m.lines.num_cells() > 5);
    }
}
