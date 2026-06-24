//! Tuning peg (guitar/violin tuning mechanism).
use crate::data::{CellArray, Points, PolyData};

pub fn tuning_peg(shaft_length: f64, head_diameter: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let shaft_r = head_diameter * 0.15;
    let head_r = head_diameter / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Shaft (cylinder)
    for s in 0..=3 {
        let z = shaft_length * s as f64 / 3.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([shaft_r * a.cos(), shaft_r * a.sin(), z]);
        }
    }
    for s in 0..3 {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Head (flat disk perpendicular to shaft)
    let _hb = pts.len();
    let hc = pts.len();
    pts.push([0.0, 0.0, shaft_length]);
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([head_r * a.cos(), 0.0, shaft_length + head_r * a.sin()]);
    }
    for j in 0..na {
        polys.push_cell(&[
            (hc) as i64,
            (hc + 1 + j) as i64,
            (hc + 1 + (j + 1) % na) as i64,
        ]);
    }
    // String hole through shaft
    let mut lines = CellArray::new();
    let hole0 = pts.len();
    pts.push([0.0, -shaft_r * 1.5, shaft_length * 0.3]);
    let hole1 = pts.len();
    pts.push([0.0, shaft_r * 1.5, shaft_length * 0.3]);
    lines.push_cell(&[hole0 as i64, hole1 as i64]);
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
    fn test_peg() {
        let m = tuning_peg(2.0, 1.0, 10);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 20);
    }
}
