//! Simple sundial with gnomon and hour lines.
use crate::data::{CellArray, Points, PolyData};

pub fn sundial(base_radius: f64, gnomon_height: f64, n_hours: usize) -> PolyData {
    let nh = n_hours.max(6);
    let na = 32;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Circular base disk
    let center = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([base_radius * a.cos(), base_radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[
            center as i64,
            (center + 1 + j) as i64,
            (center + 1 + (j + 1) % na) as i64,
        ]);
    }
    // Gnomon (triangular blade)
    let g0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let g1 = pts.len();
    pts.push([base_radius * 0.8, 0.0, 0.0]);
    let g2 = pts.len();
    pts.push([0.0, 0.0, gnomon_height]);
    polys.push_cell(&[g0 as i64, g1 as i64, g2 as i64]);
    // Hour lines
    for h in 0..nh {
        let angle =
            -std::f64::consts::PI / 2.0 + std::f64::consts::PI * h as f64 / (nh - 1).max(1) as f64;
        let lstart = pts.len();
        pts.push([0.0, 0.0, 0.001]);
        let lend = pts.len();
        pts.push([
            base_radius * 0.9 * angle.cos(),
            base_radius * 0.9 * angle.sin(),
            0.001,
        ]);
        lines.push_cell(&[lstart as i64, lend as i64]);
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
    fn test_sundial() {
        let m = sundial(5.0, 3.0, 12);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 10);
        assert!(m.lines.num_cells() >= 6);
    }
}
