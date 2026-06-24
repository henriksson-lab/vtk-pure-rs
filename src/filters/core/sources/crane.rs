//! Construction crane geometry (tower + jib + counter-jib).
use crate::data::{CellArray, Points, PolyData};
pub fn tower_crane(
    tower_height: f64,
    jib_length: f64,
    counter_length: f64,
    tower_width: f64,
) -> PolyData {
    let hw = tower_width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Tower (4 vertical lines)
    for &x in &[-hw, hw] {
        for &y in &[-hw, hw] {
            let b = pts.len();
            pts.push([x, y, 0.0]);
            pts.push([x, y, tower_height]);
            lines.push_cell(&[b as i64, (b + 1) as i64]);
        }
    }
    // Tower bracing (horizontal rings)
    let rings = 5;
    for ri in 0..=rings {
        let z = tower_height * ri as f64 / rings as f64;
        let b = pts.len();
        pts.push([-hw, -hw, z]);
        pts.push([hw, -hw, z]);
        pts.push([hw, hw, z]);
        pts.push([-hw, hw, z]);
        for i in 0..4 {
            lines.push_cell(&[(b + i) as i64, (b + (i + 1) % 4) as i64]);
        }
    }
    // Jib (horizontal boom)
    let jb = pts.len();
    pts.push([0.0, 0.0, tower_height]);
    pts.push([jib_length, 0.0, tower_height]);
    lines.push_cell(&[jb as i64, (jb + 1) as i64]);
    // Counter-jib
    let cb = pts.len();
    pts.push([0.0, 0.0, tower_height]);
    pts.push([-counter_length, 0.0, tower_height]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    // Trolley cable (vertical from jib tip)
    let tb = pts.len();
    pts.push([jib_length * 0.7, 0.0, tower_height]);
    pts.push([jib_length * 0.7, 0.0, tower_height * 0.3]);
    lines.push_cell(&[tb as i64, (tb + 1) as i64]);
    // Cat head (peak above tower)
    let peak = pts.len();
    pts.push([0.0, 0.0, tower_height * 1.1]);
    let jib_tip = jb + 1;
    let counter_tip = cb + 1;
    lines.push_cell(&[peak as i64, jib_tip as i64]);
    lines.push_cell(&[peak as i64, counter_tip as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = tower_crane(30.0, 40.0, 15.0, 2.0);
        assert!(c.lines.num_cells() > 20);
    }
}
