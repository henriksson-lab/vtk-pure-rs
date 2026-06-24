//! Kaleidoscope radial symmetry pattern.
use crate::data::{CellArray, Points, PolyData};

pub fn kaleidoscope(radius: f64, n_sectors: usize, n_rings: usize) -> PolyData {
    let ns = n_sectors.max(3);
    let nr = n_rings.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Center point
    pts.push([0.0, 0.0, 0.0]);
    for r in 1..=nr {
        let ri = radius * r as f64 / nr as f64;
        for s in 0..ns {
            let a = 2.0 * std::f64::consts::PI * s as f64 / ns as f64;
            // Add slight z variation for visual interest
            let z = 0.1 * ((r * s) as f64 * 0.5).sin();
            pts.push([ri * a.cos(), ri * a.sin(), z]);
        }
    }
    // Center fan
    for s in 0..ns {
        polys.push_cell(&[0, (1 + s) as i64, (1 + (s + 1) % ns) as i64]);
    }
    // Rings
    for r in 1..nr {
        let b0 = 1 + (r - 1) * ns;
        let b1 = 1 + r * ns;
        for s in 0..ns {
            let s1 = (s + 1) % ns;
            polys.push_cell(&[(b0 + s) as i64, (b1 + s) as i64, (b1 + s1) as i64]);
            polys.push_cell(&[(b0 + s) as i64, (b1 + s1) as i64, (b0 + s1) as i64]);
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
    fn test_kaleidoscope() {
        let m = kaleidoscope(5.0, 12, 5);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 50);
    }
}
