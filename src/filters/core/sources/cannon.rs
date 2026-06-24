//! Historical cannon with barrel and carriage.
use crate::data::{CellArray, Points, PolyData};

pub fn cannon(barrel_length: f64, barrel_radius: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Barrel (tapered cylinder)
    let nf = 6;
    for f in 0..=nf {
        let t = f as f64 / nf as f64;
        let z = barrel_length * t;
        let r = barrel_radius * (1.0 + 0.3 * (1.0 - t)); // wider at breech
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
    // Muzzle opening (ring at front)
    let bore_r = barrel_radius * 0.6;
    let muzzle_base = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([bore_r * a.cos(), bore_r * a.sin(), barrel_length]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (nf * na + j) as i64,
            (muzzle_base + j) as i64,
            (muzzle_base + j1) as i64,
            (nf * na + j1) as i64,
        ]);
    }
    // Simple carriage (two lines as axle + wheels would be complex)
    let mut lines = CellArray::new();
    let cb = pts.len();
    pts.push([-barrel_radius * 2.0, 0.0, 0.0]);
    pts.push([barrel_radius * 2.0, 0.0, 0.0]);
    pts.push([-barrel_radius * 1.5, 0.0, -barrel_radius]);
    pts.push([barrel_radius * 1.5, 0.0, -barrel_radius]);
    lines.push_cell(&[cb as i64, (cb + 1) as i64]); // axle
    lines.push_cell(&[cb as i64, (cb + 2) as i64]); // left trail
    lines.push_cell(&[(cb + 1) as i64, (cb + 3) as i64]); // right trail
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
    fn test_cannon() {
        let m = cannon(3.0, 0.3, 10);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 40);
    }
}
