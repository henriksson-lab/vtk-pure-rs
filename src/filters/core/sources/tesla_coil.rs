//! Tesla coil (primary coil, secondary coil, toroid top load).
use crate::data::{CellArray, Points, PolyData};

pub fn tesla_coil(height: f64, secondary_radius: f64, toroid_radius: f64, na: usize) -> PolyData {
    let na = na.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Secondary coil (tall cylinder)
    let nf = 8;
    for f in 0..=nf {
        let z = height * f as f64 / nf as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([secondary_radius * a.cos(), secondary_radius * a.sin(), z]);
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
    // Primary coil (spiral at base)
    let primary_r = secondary_radius * 2.5;
    let primary_turns = 5.0;
    let pn = (primary_turns * na as f64) as usize;
    let pb = pts.len();
    for i in 0..=pn {
        let t = i as f64 / pn as f64;
        let a = 2.0 * std::f64::consts::PI * primary_turns * t;
        let r = secondary_radius * 1.5 + (primary_r - secondary_radius * 1.5) * t;
        pts.push([r * a.cos(), r * a.sin(), height * 0.05]);
    }
    for i in 0..pn {
        lines.push_cell(&[(pb + i) as i64, (pb + i + 1) as i64]);
    }
    // Toroid top load
    let tor_tube_r = toroid_radius * 0.3;
    let tor_center_z = height + tor_tube_r;
    let nt = 8;
    for i in 0..na {
        let a = 2.0 * std::f64::consts::PI * i as f64 / na as f64;
        let cx = toroid_radius * a.cos();
        let cy = toroid_radius * a.sin();
        let tb = pts.len();
        for j in 0..nt {
            let b = 2.0 * std::f64::consts::PI * j as f64 / nt as f64;
            pts.push([
                cx + tor_tube_r * b.cos() * a.cos(),
                cy + tor_tube_r * b.cos() * a.sin(),
                tor_center_z + tor_tube_r * b.sin(),
            ]);
        }
        if i > 0 {
            let prev = tb - nt;
            for j in 0..nt {
                let j1 = (j + 1) % nt;
                polys.push_cell(&[(prev + j) as i64, (tb + j) as i64, (tb + j1) as i64]);
                polys.push_cell(&[(prev + j) as i64, (tb + j1) as i64, (prev + j1) as i64]);
            }
        }
    }
    // Close toroid
    let last = pts.len() - nt;
    let first_tor = last - (na - 1) * nt;
    for j in 0..nt {
        let j1 = (j + 1) % nt;
        polys.push_cell(&[
            (last + j) as i64,
            (first_tor + j) as i64,
            (first_tor + j1) as i64,
        ]);
        polys.push_cell(&[
            (last + j) as i64,
            (first_tor + j1) as i64,
            (last + j1) as i64,
        ]);
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
    fn test_tesla() {
        let m = tesla_coil(5.0, 0.3, 0.8, 16);
        assert!(m.points.len() > 200);
        assert!(m.polys.num_cells() > 150);
    }
}
