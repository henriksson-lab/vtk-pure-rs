//! Roman abacus with grooves and sliding counters.
use crate::data::{CellArray, Points, PolyData};

pub fn roman_abacus(width: f64, height: f64, n_columns: usize) -> PolyData {
    let nc = n_columns.max(5);
    let hw = width / 2.0;
    let hh = height / 2.0;
    let depth = width * 0.03;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Frame
    let fb = pts.len();
    pts.push([-hw, -depth, -hh]);
    pts.push([hw, -depth, -hh]);
    pts.push([hw, depth, -hh]);
    pts.push([-hw, depth, -hh]);
    pts.push([-hw, -depth, hh]);
    pts.push([hw, -depth, hh]);
    pts.push([hw, depth, hh]);
    pts.push([-hw, depth, hh]);
    polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 5) as i64, (fb + 4) as i64]); // front
    polys.push_cell(&[
        (fb + 4) as i64,
        (fb + 5) as i64,
        (fb + 6) as i64,
        (fb + 7) as i64,
    ]); // top
    polys.push_cell(&[fb as i64, (fb + 3) as i64, (fb + 2) as i64, (fb + 1) as i64]); // bottom
                                                                                      // Divider (horizontal bar at 1/3 from top)
    let div_z = hh * 0.33;
    let db0 = pts.len();
    pts.push([-hw, -depth - 0.001, div_z]);
    let db1 = pts.len();
    pts.push([hw, -depth - 0.001, div_z]);
    lines.push_cell(&[db0 as i64, db1 as i64]);
    // Column grooves and counters
    for c in 0..nc {
        let x = -hw * 0.8 + width * 0.8 * c as f64 / (nc - 1).max(1) as f64;
        // Groove line
        let g0 = pts.len();
        pts.push([x, -depth - 0.001, -hh * 0.9]);
        let g1 = pts.len();
        pts.push([x, -depth - 0.001, hh * 0.9]);
        lines.push_cell(&[g0 as i64, g1 as i64]);
        // Lower counters (4 beads)
        for b in 0..4 {
            let bz = -hh * 0.7 + height * 0.25 * b as f64 / 4.0;
            let bb = pts.len();
            let br = width * 0.025;
            for j in 0..6 {
                let a = 2.0 * std::f64::consts::PI * j as f64 / 6.0;
                pts.push([x + br * a.cos(), -depth - 0.002, bz + br * a.sin()]);
            }
            for j in 0..6 {
                lines.push_cell(&[(bb + j) as i64, (bb + (j + 1) % 6) as i64]);
            }
        }
        // Upper counter (1 bead)
        let ubz = div_z + height * 0.15;
        let ub = pts.len();
        let ubr = width * 0.03;
        for j in 0..6 {
            let a = 2.0 * std::f64::consts::PI * j as f64 / 6.0;
            pts.push([x + ubr * a.cos(), -depth - 0.002, ubz + ubr * a.sin()]);
        }
        for j in 0..6 {
            lines.push_cell(&[(ub + j) as i64, (ub + (j + 1) % 6) as i64]);
        }
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
    fn test_roman_abacus() {
        let m = roman_abacus(6.0, 4.0, 7);
        assert!(m.points.len() > 100);
        assert!(m.lines.num_cells() > 50);
    }
}
