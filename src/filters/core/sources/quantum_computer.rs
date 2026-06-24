//! Quantum computer (dilution refrigerator + qubit chip).
use crate::data::{CellArray, Points, PolyData};
pub fn quantum_computer(
    num_stages: usize,
    top_r: f64,
    bottom_r: f64,
    height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let ns = num_stages.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Dilution refrigerator stages (nested cylinders, getting smaller)
    let stage_h = height / ns as f64;
    for si in 0..ns {
        let t = si as f64 / (ns - 1) as f64;
        let r = top_r * (1.0 - t) + bottom_r * t;
        let z = si as f64 * stage_h;
        // Stage plate (disk)
        let pc = pts.len();
        pts.push([0.0, 0.0, z]);
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
        for i in 0..res {
            let j = if i + 1 < res { pc + 2 + i } else { pc + 1 };
            polys.push_cell(&[pc as i64, (pc + 1 + i) as i64, j as i64]);
        }
        // Support rods to next stage
        if si < ns - 1 {
            let next_r = top_r * (1.0 - (si + 1) as f64 / (ns - 1) as f64)
                + bottom_r * (si + 1) as f64 / (ns - 1) as f64;
            for ri in 0..4 {
                let a = std::f64::consts::FRAC_PI_2 * ri as f64;
                let rb = pts.len();
                pts.push([r * 0.8 * a.cos(), r * 0.8 * a.sin(), z]);
                pts.push([next_r * 0.8 * a.cos(), next_r * 0.8 * a.sin(), z + stage_h]);
                lines.push_cell(&[rb as i64, (rb + 1) as i64]);
            }
        }
    }
    // Qubit chip (small square at bottom)
    let chip_s = bottom_r * 0.3;
    let chip_z = height;
    let cb = pts.len();
    pts.push([-chip_s, -chip_s, chip_z]);
    pts.push([chip_s, -chip_s, chip_z]);
    pts.push([chip_s, chip_s, chip_z]);
    pts.push([-chip_s, chip_s, chip_z]);
    polys.push_cell(&[cb as i64, (cb + 1) as i64, (cb + 2) as i64, (cb + 3) as i64]);
    // Wiring loom (lines from chip to top)
    for wi in 0..8 {
        let a = 2.0 * std::f64::consts::PI * wi as f64 / 8.0;
        let wr = chip_s * 0.5;
        let wb = pts.len();
        pts.push([wr * a.cos(), wr * a.sin(), chip_z]);
        pts.push([top_r * 0.3 * a.cos(), top_r * 0.3 * a.sin(), 0.0]);
        lines.push_cell(&[wb as i64, (wb + 1) as i64]);
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
        let q = quantum_computer(5, 2.0, 0.5, 3.0, 8);
        assert!(q.polys.num_cells() > 5);
        assert!(q.lines.num_cells() > 10);
    }
}
