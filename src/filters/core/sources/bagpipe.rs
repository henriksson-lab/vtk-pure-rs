//! Bagpipe (bag, chanter, drones, blowpipe).
use crate::data::{CellArray, Points, PolyData};

pub fn bagpipe(bag_radius: f64, na: usize) -> PolyData {
    let na = na.max(10);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Bag (sphere)
    let np = 4;
    let top = pts.len();
    pts.push([0.0, 0.0, bag_radius]);
    for p in 1..np {
        let phi = std::f64::consts::PI * p as f64 / np as f64;
        let r = bag_radius * phi.sin();
        let z = bag_radius * phi.cos();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for j in 0..na {
        polys.push_cell(&[
            top as i64,
            (top + 1 + j) as i64,
            (top + 1 + (j + 1) % na) as i64,
        ]);
    }
    for p in 0..(np - 2) {
        let b0 = top + 1 + p * na;
        let b1 = top + 1 + (p + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Chanter (long pipe downward)
    let ch0 = pts.len();
    pts.push([bag_radius * 0.8, 0.0, -bag_radius * 0.5]);
    let ch1 = pts.len();
    pts.push([bag_radius * 1.2, 0.0, -bag_radius * 2.5]);
    lines.push_cell(&[ch0 as i64, ch1 as i64]);
    // Drones (3 pipes upward)
    for i in 0..3 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0 + 0.5;
        let dx = bag_radius * 0.3 * a.cos();
        let dy = bag_radius * 0.3 * a.sin();
        let d0 = pts.len();
        pts.push([dx, dy, bag_radius * 0.5]);
        let d1 = pts.len();
        pts.push([dx, dy, bag_radius * (1.5 + 0.5 * i as f64)]);
        lines.push_cell(&[d0 as i64, d1 as i64]);
    }
    // Blowpipe
    let bp0 = pts.len();
    pts.push([-bag_radius * 0.7, 0.0, 0.0]);
    let bp1 = pts.len();
    pts.push([-bag_radius * 2.0, 0.0, bag_radius * 0.3]);
    lines.push_cell(&[bp0 as i64, bp1 as i64]);
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
    fn test_bagpipe() {
        let m = bagpipe(1.0, 12);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 20);
        assert!(m.lines.num_cells() >= 5);
    }
}
