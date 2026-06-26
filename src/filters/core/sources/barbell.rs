//! Weightlifting barbell with plates.
use crate::data::{CellArray, Points, PolyData};

pub fn barbell(
    bar_length: f64,
    bar_radius: f64,
    plate_radius: f64,
    n_plates_per_side: usize,
    na: usize,
) -> PolyData {
    let na = na.max(8);
    let np = n_plates_per_side.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Bar (cylinder along X)
    let bar_segs = 6;
    for s in 0..=bar_segs {
        let x = -bar_length / 2.0 + bar_length * s as f64 / bar_segs as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x, bar_radius * a.cos(), bar_radius * a.sin()]);
        }
    }
    for s in 0..bar_segs {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Plates on each side
    let plate_thick = bar_length * 0.02;
    for &side in &[-1.0f64, 1.0] {
        for p in 0..np {
            let cx = side * (bar_length / 2.0 - 0.1 - plate_thick * (p as f64 + 0.5));
            let x0 = cx - plate_thick / 2.0;
            let x1 = cx + plate_thick / 2.0;
            let c0 = pts.len();
            pts.push([x0, 0.0, 0.0]);
            let r0 = pts.len();
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([x0, plate_radius * a.cos(), plate_radius * a.sin()]);
            }
            let c1 = pts.len();
            pts.push([x1, 0.0, 0.0]);
            let r1 = pts.len();
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([x1, plate_radius * a.cos(), plate_radius * a.sin()]);
            }
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[c0 as i64, (r0 + j1) as i64, (r0 + j) as i64]);
                polys.push_cell(&[c1 as i64, (r1 + j) as i64, (r1 + j1) as i64]);
                polys.push_cell(&[
                    (r0 + j) as i64,
                    (r0 + j1) as i64,
                    (r1 + j1) as i64,
                    (r1 + j) as i64,
                ]);
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
    fn test_barbell() {
        let m = barbell(2.2, 0.025, 0.225, 2, 10);
        assert!(m.points.len() > 70);
        assert!(m.polys.num_cells() > 60);
    }
}
