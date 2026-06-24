//! Deployable satellite antenna (umbrella-style reflector).
use crate::data::{CellArray, Points, PolyData};

pub fn satellite_antenna(
    diameter: f64,
    focal_length: f64,
    n_ribs: usize,
    n_rings: usize,
) -> PolyData {
    let nr = n_ribs.max(6);
    let nrings = n_rings.max(3);
    let r_max = diameter / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Hub
    pts.push([0.0, 0.0, 0.0]);
    // Reflector surface
    for ring in 1..=nrings {
        let r = r_max * ring as f64 / nrings as f64;
        let z = r * r / (4.0 * focal_length);
        for rib in 0..nr {
            let a = 2.0 * std::f64::consts::PI * rib as f64 / nr as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    // Top cap triangles
    for j in 0..nr {
        polys.push_cell(&[0, (1 + j) as i64, (1 + (j + 1) % nr) as i64]);
    }
    // Ring quads
    for ring in 0..(nrings - 1) {
        let b0 = 1 + ring * nr;
        let b1 = 1 + (ring + 1) * nr;
        for j in 0..nr {
            let j1 = (j + 1) % nr;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Rib lines
    for rib in 0..nr {
        let _a = 2.0 * std::f64::consts::PI * rib as f64 / nr as f64;
        for ring in 0..nrings {
            let idx = if ring == 0 {
                0
            } else {
                1 + (ring - 1) * nr + rib
            };
            let next = 1 + ring * nr + rib;
            lines.push_cell(&[idx as i64, next as i64]);
        }
    }
    // Feed struts
    let feed = pts.len();
    pts.push([0.0, 0.0, focal_length]);
    for k in 0..3 {
        let rim = 1 + (nrings - 1) * nr + k * nr / 3;
        lines.push_cell(&[rim as i64, feed as i64]);
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
    fn test_sat_ant() {
        let m = satellite_antenna(4.0, 2.0, 8, 4);
        assert!(m.points.len() > 25);
        assert!(m.polys.num_cells() > 20);
    }
}
