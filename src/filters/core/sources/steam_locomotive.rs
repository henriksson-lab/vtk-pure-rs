//! Simplified steam locomotive (boiler, cab, wheels, smokestack).
use crate::data::{CellArray, Points, PolyData};

pub fn steam_locomotive(length: f64, na: usize) -> PolyData {
    let na = na.max(10);
    let boiler_r = length * 0.12;
    let boiler_len = length * 0.6;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Boiler (horizontal cylinder)
    let nf = 4;
    for f in 0..=nf {
        let x = boiler_len * f as f64 / nf as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x, boiler_r * a.cos(), boiler_r + boiler_r * a.sin()]);
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
    // Cab (box at rear)
    let cab_w = boiler_r * 1.3;
    let cab_h = boiler_r * 1.8;
    let cab_len = length * 0.25;
    let cb = pts.len();
    pts.push([boiler_len, -cab_w, 0.0]);
    pts.push([boiler_len + cab_len, -cab_w, 0.0]);
    pts.push([boiler_len + cab_len, cab_w, 0.0]);
    pts.push([boiler_len, cab_w, 0.0]);
    pts.push([boiler_len, -cab_w, cab_h]);
    pts.push([boiler_len + cab_len, -cab_w, cab_h]);
    pts.push([boiler_len + cab_len, cab_w, cab_h]);
    pts.push([boiler_len, cab_w, cab_h]);
    polys.push_cell(&[
        (cb) as i64,
        (cb + 1) as i64,
        (cb + 5) as i64,
        (cb + 4) as i64,
    ]);
    polys.push_cell(&[
        (cb + 1) as i64,
        (cb + 2) as i64,
        (cb + 6) as i64,
        (cb + 5) as i64,
    ]);
    polys.push_cell(&[
        (cb + 2) as i64,
        (cb + 3) as i64,
        (cb + 7) as i64,
        (cb + 6) as i64,
    ]);
    polys.push_cell(&[
        (cb + 4) as i64,
        (cb + 5) as i64,
        (cb + 6) as i64,
        (cb + 7) as i64,
    ]); // roof
        // Smokestack (small cylinder at front top)
    let stack_r = boiler_r * 0.25;
    let stack_h = boiler_r * 1.5;
    let stack_x = boiler_len * 0.15;
    let sb = pts.len();
    for s in 0..=2 {
        let z = 2.0 * boiler_r + stack_h * s as f64 / 2.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([stack_x + stack_r * a.cos(), stack_r * a.sin(), z]);
        }
    }
    for s in 0..2 {
        let b0 = sb + s * na;
        let b1 = sb + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (b0 + j) as i64,
                (b1 + j) as i64,
                (b1 + j1) as i64,
                (b0 + j1) as i64,
            ]);
        }
    }
    // Wheels (circles on each side)
    let wheel_r = boiler_r * 0.5;
    for &y_side in &[-boiler_r * 1.1, boiler_r * 1.1] {
        for &wx in &[boiler_len * 0.2, boiler_len * 0.5, boiler_len * 0.8] {
            let wb = pts.len();
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([wx + wheel_r * a.cos(), y_side, wheel_r + wheel_r * a.sin()]);
            }
            for j in 0..na {
                lines.push_cell(&[(wb + j) as i64, (wb + (j + 1) % na) as i64]);
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
    fn test_locomotive() {
        let m = steam_locomotive(10.0, 12);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 50);
        assert!(m.lines.num_cells() > 30);
    }
}
