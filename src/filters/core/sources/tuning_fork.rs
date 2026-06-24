//! Tuning fork (U-shaped prong with handle).
use crate::data::{CellArray, Points, PolyData};

pub fn tuning_fork(prong_length: f64, prong_gap: f64, handle_length: f64, na: usize) -> PolyData {
    let na = na.max(6);
    let prong_r = prong_gap * 0.12;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Handle (cylinder along -Z)
    let handle_r = prong_gap * 0.15;
    let hs = 3;
    for s in 0..=hs {
        let z = -handle_length * s as f64 / hs as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([handle_r * a.cos(), handle_r * a.sin(), z]);
        }
    }
    for s in 0..hs {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // U-bend at top (semicircle connecting two prongs)
    let _bend_na = 8;
    let _bend_r = prong_gap / 2.0;
    for &x_side in &[-1.0f64, 1.0] {
        // Prong (cylinder along +Z)
        let pb = pts.len();
        let ps = 4;
        for s in 0..=ps {
            let z = prong_length * s as f64 / ps as f64;
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([
                    x_side * prong_gap / 2.0 + prong_r * a.cos(),
                    prong_r * a.sin(),
                    z,
                ]);
            }
        }
        for s in 0..ps {
            let b0 = pb + s * na;
            let b1 = pb + (s + 1) * na;
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
            }
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
    fn test_tuning_fork() {
        let m = tuning_fork(5.0, 1.0, 3.0, 8);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 60);
    }
}
