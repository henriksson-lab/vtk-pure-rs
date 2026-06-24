//! Cyclotron particle accelerator geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn cyclotron(dee_r: f64, dee_gap: f64, magnet_h: f64, resolution: usize) -> PolyData {
    let res = resolution.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Two D-shaped electrodes (dees)
    for &side in &[0.0f64, std::f64::consts::PI] {
        let db = pts.len();
        // Semicircular dee
        for iz in 0..=1 {
            let z = if iz == 0 {
                -dee_gap / 2.0
            } else {
                dee_gap / 2.0
            };
            for i in 0..=res / 2 {
                let a = side + std::f64::consts::PI * i as f64 / (res / 2) as f64;
                pts.push([dee_r * a.cos(), dee_r * a.sin(), z]);
            }
        }
        let w = res / 2 + 1;
        for i in 0..res / 2 {
            polys.push_cell(&[
                (db + i) as i64,
                (db + i + 1) as i64,
                (db + w + i + 1) as i64,
                (db + w + i) as i64,
            ]);
        }
    }
    // Magnet poles (flat disks above and below)
    let mag_r = dee_r * 1.2;
    for &z in &[-dee_gap / 2.0 - magnet_h, dee_gap / 2.0 + magnet_h] {
        let mc = pts.len();
        pts.push([0.0, 0.0, z]);
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([mag_r * a.cos(), mag_r * a.sin(), z]);
        }
        for i in 0..res {
            let j = if i + 1 < res { mc + 2 + i } else { mc + 1 };
            polys.push_cell(&[mc as i64, (mc + 1 + i) as i64, j as i64]);
        }
    }
    // Spiral beam path
    let spiral_turns = 5;
    let spiral_steps = res * spiral_turns;
    let mut beam_ids = Vec::new();
    for i in 0..=spiral_steps {
        let t = i as f64 / spiral_steps as f64;
        let r = dee_r * 0.1 + dee_r * 0.8 * t;
        let a = 2.0 * std::f64::consts::PI * spiral_turns as f64 * t;
        let idx = pts.len();
        pts.push([r * a.cos(), r * a.sin(), 0.0]);
        beam_ids.push(idx as i64);
    }
    lines.push_cell(&beam_ids);
    // Extraction channel
    let eb = pts.len();
    pts.push([dee_r * 0.9, 0.0, 0.0]);
    pts.push([dee_r * 1.5, 0.0, 0.0]);
    lines.push_cell(&[eb as i64, (eb + 1) as i64]);
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
        let c = cyclotron(5.0, 0.5, 1.0, 16);
        assert!(c.polys.num_cells() > 20);
        assert!(c.lines.num_cells() >= 2);
    }
}
