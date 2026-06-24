//! Kalimba (thumb piano / mbira).
use crate::data::{CellArray, Points, PolyData};

pub fn kalimba(width: f64, height: f64, n_tines: usize) -> PolyData {
    let nt = n_tines.max(5);
    let hw = width / 2.0;
    let hh = height / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Body (rounded rectangle approximated as box)
    let depth = width * 0.15;
    let bb = pts.len();
    pts.push([-hw, -depth / 2.0, -hh]);
    pts.push([hw, -depth / 2.0, -hh]);
    pts.push([hw, depth / 2.0, -hh]);
    pts.push([-hw, depth / 2.0, -hh]);
    pts.push([-hw, -depth / 2.0, hh]);
    pts.push([hw, -depth / 2.0, hh]);
    pts.push([hw, depth / 2.0, hh]);
    pts.push([-hw, depth / 2.0, hh]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 5) as i64, (bb + 4) as i64]);
    polys.push_cell(&[
        (bb + 2) as i64,
        (bb + 3) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
    ]);
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 7) as i64,
    ]); // top
    polys.push_cell(&[bb as i64, (bb + 3) as i64, (bb + 2) as i64, (bb + 1) as i64]); // bottom
                                                                                      // Sound hole (circle on top face)
    let hole_r = width * 0.12;
    let hole_y = -depth / 2.0 - 0.01;
    let na = 12;
    let hb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([hole_r * a.cos(), hole_y, -hh * 0.3 + hole_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(hb + j) as i64, (hb + (j + 1) % na) as i64]);
    }
    // Tines (metal strips of varying length)
    let bridge_z = hh * 0.3;
    for i in 0..nt {
        let x = -hw * 0.7 + width * 0.7 * i as f64 / (nt - 1).max(1) as f64;
        // Length pattern: longest in center, shorter at edges
        let center_dist = ((i as f64 - (nt - 1) as f64 / 2.0) / (nt as f64 / 2.0)).abs();
        let tine_len = height * 0.3 * (1.0 - 0.4 * center_dist);
        let t0 = pts.len();
        pts.push([x, -depth / 2.0 - 0.01, bridge_z]);
        let t1 = pts.len();
        pts.push([x, -depth / 2.0 - 0.01, bridge_z + tine_len]);
        lines.push_cell(&[t0 as i64, t1 as i64]);
    }
    // Bridge bar
    let br0 = pts.len();
    pts.push([-hw * 0.8, -depth / 2.0 - 0.01, bridge_z]);
    let br1 = pts.len();
    pts.push([hw * 0.8, -depth / 2.0 - 0.01, bridge_z]);
    lines.push_cell(&[br0 as i64, br1 as i64]);
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
    fn test_kalimba() {
        let m = kalimba(3.0, 5.0, 9);
        assert!(m.points.len() > 25);
        assert!(m.polys.num_cells() >= 4);
        assert!(m.lines.num_cells() > 10);
    }
}
