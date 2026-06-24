//! Hourglass (two cones meeting at a narrow waist).
use crate::data::{CellArray, Points, PolyData};

pub fn hourglass(
    height: f64,
    radius: f64,
    waist_ratio: f64,
    n_floors: usize,
    n_angular: usize,
) -> PolyData {
    let nf = n_floors.max(4);
    let na = n_angular.max(8);
    let waist = radius * waist_ratio.clamp(0.01, 0.5);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for f in 0..=nf {
        let t = f as f64 / nf as f64;
        let z = height * t;
        // Radius: wide at top and bottom, narrow at middle
        let r = if t < 0.5 {
            radius - (radius - waist) * (t * 2.0)
        } else {
            waist + (radius - waist) * ((t - 0.5) * 2.0)
        };
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for f in 0..nf {
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
    fn test_hourglass() {
        let m = hourglass(10.0, 3.0, 0.2, 8, 12);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 40);
    }
}
