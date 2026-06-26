//! Ancient Greek lyre (harp-like instrument).
use crate::data::{CellArray, Points, PolyData};

pub fn lyre(height: f64, width: f64, n_strings: usize) -> PolyData {
    let ns = n_strings.max(3);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Sound box (bottom bar)
    let sb0 = pts.len();
    pts.push([-hw, 0.0, 0.0]);
    let sb1 = pts.len();
    pts.push([hw, 0.0, 0.0]);
    lines.push_cell(&[sb0 as i64, sb1 as i64]);
    // Arms (curved upward)
    let na = 10;
    let mut arm_tips = Vec::new();
    for &(side, mut previous) in &[(-1.0f64, sb0), (1.0, sb1)] {
        for j in 1..=na {
            let t = j as f64 / na as f64;
            let x = side * hw * (1.0 + 0.2 * (std::f64::consts::PI * t * 0.5).sin());
            let z = height * t;
            let current = pts.len();
            pts.push([x, 0.0, z]);
            lines.push_cell(&[previous as i64, current as i64]);
            previous = current;
        }
        arm_tips.push(previous);
    }
    // Crossbar at top
    lines.push_cell(&[arm_tips[0] as i64, arm_tips[1] as i64]);
    // Strings
    for si in 0..ns {
        let t = (si + 1) as f64 / (ns + 1) as f64;
        let x = -hw + width * t;
        let s_bottom = pts.len();
        pts.push([x, 0.0, height * 0.05]);
        let s_top = pts.len();
        pts.push([x * 1.1, 0.0, height * 0.95]);
        lines.push_cell(&[s_bottom as i64, s_top as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lyre() {
        let m = lyre(5.0, 3.0, 7);
        assert!(m.points.len() > 20);
        assert!(m.lines.num_cells() > 10);
    }
}
