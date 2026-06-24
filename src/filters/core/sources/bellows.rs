//! Bellows (accordion-fold flexible joint).
use crate::data::{CellArray, Points, PolyData};

pub fn bellows(radius: f64, length: f64, n_folds: usize, na: usize) -> PolyData {
    let nf = n_folds.max(2);
    let na = na.max(8);
    let fold_len = length / nf as f64;
    let inner_r = radius * 0.7;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Alternating wide and narrow rings
    for f in 0..=nf * 2 {
        let z = fold_len * f as f64 / 2.0;
        let r = if f % 2 == 0 { radius } else { inner_r };
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    let total_rings = nf * 2 + 1;
    for f in 0..(total_rings - 1) {
        let b0 = f * na;
        let b1 = (f + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bellows() {
        let m = bellows(1.0, 3.0, 5, 12);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 100);
    }
}
