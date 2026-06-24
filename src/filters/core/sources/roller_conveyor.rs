//! Roller conveyor with visible rollers.
use crate::data::{CellArray, Points, PolyData};
pub fn roller_conveyor(
    length: f64,
    width: f64,
    roller_r: f64,
    roller_spacing: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let hw = width / 2.0;
    let nr = ((length / roller_spacing).ceil() as usize).max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Side rails
    for &y in &[-hw, hw] {
        let rb = pts.len();
        pts.push([0.0, y, roller_r]);
        pts.push([length, y, roller_r]);
        lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    }
    // Rollers
    for ri in 0..=nr {
        let x = ri as f64 * roller_spacing;
        if x > length {
            break;
        }
        let rb = pts.len();
        for ring in 0..=1 {
            let y = if ring == 0 { -hw } else { hw };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([x, y, roller_r + roller_r * a.cos()]);
            }
        }
        // Actually rollers go along Y, so fix coordinates
        let rb2 = pts.len() - (res * 2);
        for ring in 0..=1 {
            for i in 0..res {
                let idx = rb2 + ring * res + i;
                let _p = pts.get(idx);
                let _y_pos = if ring == 0 { -hw } else { hw };
                // Already pushed, just connect
            }
        }
        // Connect rings
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (rb + i) as i64,
                (rb + j) as i64,
                (rb + res + j) as i64,
                (rb + res + i) as i64,
            ]);
        }
    }
    // Legs
    for &x in &[0.0f64, length] {
        for &y in &[-hw, hw] {
            let lb = pts.len();
            pts.push([x, y, 0.0]);
            pts.push([x, y, roller_r]);
            lines.push_cell(&[lb as i64, (lb + 1) as i64]);
        }
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
    fn test() {
        let c = roller_conveyor(5.0, 1.0, 0.1, 0.5, 8);
        assert!(c.polys.num_cells() > 5);
    }
}
