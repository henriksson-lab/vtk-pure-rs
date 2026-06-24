//! Cassegrain reflecting telescope (primary + secondary mirrors in tube).
use crate::data::{CellArray, Points, PolyData};

pub fn cassegrain(
    tube_length: f64,
    primary_diameter: f64,
    secondary_diameter: f64,
    na: usize,
) -> PolyData {
    let na = na.max(12);
    let pr = primary_diameter / 2.0;
    let sr = secondary_diameter / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Tube (cylinder)
    let tube_r = pr * 1.1;
    let nf = 4;
    for f in 0..=nf {
        let z = tube_length * f as f64 / nf as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([tube_r * a.cos(), tube_r * a.sin(), z]);
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
    // Primary mirror (parabolic dish at z=0)
    let pmc = pts.len();
    pts.push([0.0, 0.0, -pr * pr / (4.0 * tube_length)]);
    let pmb = pts.len();
    let nr = 4;
    for r in 1..=nr {
        let rr = pr * r as f64 / nr as f64;
        let z = rr * rr / (4.0 * tube_length);
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([rr * a.cos(), rr * a.sin(), z]);
        }
    }
    for j in 0..na {
        polys.push_cell(&[pmc as i64, (pmb + j) as i64, (pmb + (j + 1) % na) as i64]);
    }
    for r in 0..(nr - 1) {
        let b0 = pmb + r * na;
        let b1 = pmb + (r + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Secondary mirror (small disk at tube_length * 0.7)
    let sec_z = tube_length * 0.7;
    let sc = pts.len();
    pts.push([0.0, 0.0, sec_z]);
    let sb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([sr * a.cos(), sr * a.sin(), sec_z]);
    }
    for j in 0..na {
        polys.push_cell(&[sc as i64, (sb + j) as i64, (sb + (j + 1) % na) as i64]);
    }
    // Spider vanes (4 lines holding secondary)
    for k in 0..4 {
        let a = std::f64::consts::PI / 2.0 * k as f64;
        let vane_outer = pts.len();
        pts.push([tube_r * a.cos(), tube_r * a.sin(), sec_z]);
        lines.push_cell(&[sc as i64, vane_outer as i64]);
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
    fn test_cassegrain() {
        let m = cassegrain(5.0, 2.0, 0.5, 16);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 60);
        assert_eq!(m.lines.num_cells(), 4);
    }
}
