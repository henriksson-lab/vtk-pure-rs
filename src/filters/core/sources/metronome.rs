//! Metronome (pyramid case with pendulum arm).
use crate::data::{CellArray, Points, PolyData};

pub fn metronome(height: f64, base_width: f64) -> PolyData {
    let hw = base_width / 2.0;
    let hd = base_width * 0.3;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pyramidal case
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    let top_w = hw * 0.3;
    let top_d = hd * 0.3;
    pts.push([-top_w, -top_d, height]);
    pts.push([top_w, -top_d, height]);
    pts.push([top_w, top_d, height]);
    pts.push([-top_w, top_d, height]);
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    polys.push_cell(&[4, 5, 6, 7]); // top
    polys.push_cell(&[0, 3, 2, 1]); // bottom
                                    // Pendulum arm (from pivot near top, swinging)
    let pivot = pts.len();
    pts.push([0.0, -hd - 0.01, height * 0.85]);
    let arm_end = pts.len();
    pts.push([hw * 0.3, -hd - 0.01, height * 0.2]);
    lines.push_cell(&[pivot as i64, arm_end as i64]);
    // Weight on arm
    let weight = pts.len();
    pts.push([hw * 0.2, -hd - 0.01, height * 0.5]);
    lines.push_cell(&[arm_end as i64, weight as i64]);
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
    fn test_metronome() {
        let m = metronome(6.0, 3.0);
        assert!(m.points.len() > 8);
        assert!(m.polys.num_cells() == 6);
    }
}
