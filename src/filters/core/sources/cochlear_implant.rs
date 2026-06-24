//! Cochlear implant (electrode array spiral).
use crate::data::{CellArray, Points, PolyData};
pub fn cochlear_implant(
    spiral_r: f64,
    turns: f64,
    num_electrodes: usize,
    electrode_r: f64,
    _wire_r: f64,
    resolution: usize,
) -> PolyData {
    let _res = resolution.max(4);
    let ne = num_electrodes.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Carrier wire (spiral)
    let total_steps = ne * 4;
    let mut wire_ids = Vec::new();
    for i in 0..=total_steps {
        let t = i as f64 / total_steps as f64;
        let a = 2.0 * std::f64::consts::PI * turns * t;
        let r = spiral_r * (1.0 - t * 0.6);
        let idx = pts.len();
        pts.push([r * a.cos(), r * a.sin(), t * spiral_r * 0.3]);
        wire_ids.push(idx as i64);
    }
    lines.push_cell(&wire_ids);
    // Electrodes (small spheroids along the spiral)
    for ei in 0..ne {
        let t = (ei as f64 + 0.5) / ne as f64;
        let a = 2.0 * std::f64::consts::PI * turns * t;
        let r = spiral_r * (1.0 - t * 0.6);
        let ex = r * a.cos();
        let ey = r * a.sin();
        let ez = t * spiral_r * 0.3;
        let eb = pts.len();
        pts.push([ex + electrode_r, ey, ez]);
        pts.push([ex - electrode_r, ey, ez]);
        pts.push([ex, ey + electrode_r, ez]);
        pts.push([ex, ey - electrode_r, ez]);
        pts.push([ex, ey, ez + electrode_r]);
        pts.push([ex, ey, ez - electrode_r]);
        let faces = [
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [0, 5, 2],
            [2, 5, 1],
            [1, 5, 3],
            [3, 5, 0],
        ];
        for f in &faces {
            polys.push_cell(&[(eb + f[0]) as i64, (eb + f[1]) as i64, (eb + f[2]) as i64]);
        }
    }
    // Receiver/stimulator (box at base)
    let rb = pts.len();
    let rs = spiral_r * 0.4;
    pts.push([-rs, -rs, -rs]);
    pts.push([rs, -rs, -rs]);
    pts.push([rs, rs, -rs]);
    pts.push([-rs, rs, -rs]);
    pts.push([-rs, -rs, 0.0]);
    pts.push([rs, -rs, 0.0]);
    pts.push([rs, rs, 0.0]);
    pts.push([-rs, rs, 0.0]);
    let f = |i: usize| (rb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result.lines = lines;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = cochlear_implant(0.005, 2.5, 16, 0.0005, 0.0002, 6);
        assert!(c.polys.num_cells() > 100);
        assert!(c.lines.num_cells() >= 1);
    }
}
