//! Binoculars (two parallel tubes with bridge).
use crate::data::{CellArray, Points, PolyData};

pub fn binoculars(tube_length: f64, tube_radius: f64, separation: f64, na: usize) -> PolyData {
    let na = na.max(10);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let hs = separation / 2.0;
    // Two tubes
    for &cy in &[-hs, hs] {
        let base = pts.len();
        let nf = 4;
        for f in 0..=nf {
            let z = tube_length * f as f64 / nf as f64;
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([tube_radius * a.cos(), cy + tube_radius * a.sin(), z]);
            }
        }
        for f in 0..nf {
            let b0 = base + f * na;
            let b1 = base + (f + 1) * na;
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
            }
        }
    }
    // Bridge connecting tubes
    let bridge_z = tube_length * 0.5;
    let bb = pts.len();
    let bw = tube_radius * 0.5;
    let bh = tube_radius * 0.4;
    pts.push([bw, -hs + tube_radius, bridge_z - bh]);
    pts.push([bw, hs - tube_radius, bridge_z - bh]);
    pts.push([bw, hs - tube_radius, bridge_z + bh]);
    pts.push([bw, -hs + tube_radius, bridge_z + bh]);
    pts.push([-bw, -hs + tube_radius, bridge_z - bh]);
    pts.push([-bw, hs - tube_radius, bridge_z - bh]);
    pts.push([-bw, hs - tube_radius, bridge_z + bh]);
    pts.push([-bw, -hs + tube_radius, bridge_z + bh]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
        (bb + 5) as i64,
    ]);
    polys.push_cell(&[
        (bb + 3) as i64,
        (bb + 2) as i64,
        (bb + 6) as i64,
        (bb + 7) as i64,
    ]);
    polys.push_cell(&[bb as i64, (bb + 4) as i64, (bb + 5) as i64, (bb + 1) as i64]);
    polys.push_cell(&[bb as i64, (bb + 3) as i64, (bb + 7) as i64, (bb + 4) as i64]);
    polys.push_cell(&[
        (bb + 1) as i64,
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 2) as i64,
    ]);
    // Focus wheel
    let fw_r = tube_radius * 0.6;
    let fwb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([fw_r * a.cos(), 0.0, bridge_z + fw_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(fwb + j) as i64, (fwb + (j + 1) % na) as i64]);
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
    fn test_binoculars() {
        let m = binoculars(4.0, 0.3, 1.2, 10);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 60);
    }
}
