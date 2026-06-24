//! Printed circuit board geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn pcb(width: f64, length: f64, thickness: f64, num_components: usize, seed: u64) -> PolyData {
    let nc = num_components.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hw = width / 2.0;
    let hl = length / 2.0;
    let ab = |pts: &mut Points<f64>,
              polys: &mut CellArray,
              x0: f64,
              y0: f64,
              z0: f64,
              x1: f64,
              y1: f64,
              z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    // Board
    ab(&mut pts, &mut polys, -hw, -hl, 0.0, hw, hl, thickness);
    // Components (random small boxes on top)
    let mut rng = seed;
    let next_f = |rng: &mut u64| -> f64 {
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((*rng >> 33) as f64) / (u32::MAX as f64)
    };
    for _ in 0..nc {
        let cx = -hw * 0.8 + next_f(&mut rng) * width * 0.8;
        let cy = -hl * 0.8 + next_f(&mut rng) * length * 0.8;
        let cw = width * 0.02 + next_f(&mut rng) * width * 0.08;
        let cl = length * 0.02 + next_f(&mut rng) * length * 0.05;
        let ch = thickness + next_f(&mut rng) * thickness * 3.0;
        ab(
            &mut pts,
            &mut polys,
            cx,
            cy,
            thickness,
            cx + cw,
            cy + cl,
            ch,
        );
    }
    // Traces (random lines on board surface)
    for _ in 0..nc * 2 {
        let x0 = -hw * 0.9 + next_f(&mut rng) * width * 0.9;
        let y0 = -hl * 0.9 + next_f(&mut rng) * length * 0.9;
        let x1 = -hw * 0.9 + next_f(&mut rng) * width * 0.9;
        let y1 = -hl * 0.9 + next_f(&mut rng) * length * 0.9;
        let tb = pts.len();
        pts.push([x0, y0, thickness + 0.0001]);
        pts.push([x1, y1, thickness + 0.0001]);
        lines.push_cell(&[tb as i64, (tb + 1) as i64]);
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
        let p = pcb(0.1, 0.08, 0.002, 10, 42);
        assert!(p.polys.num_cells() > 60);
        assert!(p.lines.num_cells() >= 10);
    }
}
