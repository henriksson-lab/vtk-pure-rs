//! Abacus (counting frame with beads on rods).
use crate::data::{CellArray, Points, PolyData};

pub fn abacus(width: f64, height: f64, n_rods: usize, beads_per_rod: usize, na: usize) -> PolyData {
    let nr = n_rods.max(3);
    let nb = beads_per_rod.max(3);
    let na = na.max(6);
    let hw = width / 2.0;
    let hh = height / 2.0;
    let mut pts = Points::<f64>::new();
    let polys = CellArray::new();
    let mut lines = CellArray::new();
    // Frame
    let f0 = pts.len();
    pts.push([-hw, 0.0, -hh]);
    let f1 = pts.len();
    pts.push([hw, 0.0, -hh]);
    let f2 = pts.len();
    pts.push([hw, 0.0, hh]);
    let f3 = pts.len();
    pts.push([-hw, 0.0, hh]);
    lines.push_cell(&[f0 as i64, f1 as i64]);
    lines.push_cell(&[f1 as i64, f2 as i64]);
    lines.push_cell(&[f2 as i64, f3 as i64]);
    lines.push_cell(&[f3 as i64, f0 as i64]);
    // Rods and beads
    let bead_r = (height / (nb as f64 * 3.0)).min(width / (nr as f64 * 4.0));
    for ri in 0..nr {
        let x = -hw + width * (ri + 1) as f64 / (nr + 1) as f64;
        // Rod
        let r0 = pts.len();
        pts.push([x, 0.0, -hh]);
        let r1 = pts.len();
        pts.push([x, 0.0, hh]);
        lines.push_cell(&[r0 as i64, r1 as i64]);
        // Beads (small toroids approximated as rings)
        for bi in 0..nb {
            let z = -hh + height * (bi + 1) as f64 / (nb + 1) as f64;
            let bb = pts.len();
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([x + bead_r * a.cos(), bead_r * a.sin(), z]);
            }
            for j in 0..na {
                let j1 = (j + 1) % na;
                lines.push_cell(&[(bb + j) as i64, (bb + j1) as i64]);
            }
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
    fn test_abacus() {
        let m = abacus(6.0, 4.0, 5, 7, 8);
        assert!(m.points.len() > 50);
        assert!(m.lines.num_cells() > 30);
    }
}
