//! Medieval siege tower with platforms and ramp.
use crate::data::{CellArray, Points, PolyData};

pub fn siege_tower(width: f64, height: f64, n_floors: usize) -> PolyData {
    let hw = width / 2.0;
    let hd = width * 0.4;
    let nf = n_floors.max(2);
    let floor_h = height / nf as f64;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Corner posts
    for &x in &[-hw, hw] {
        for &y in &[-hd, hd] {
            let b = pts.len();
            pts.push([x, y, 0.0]);
            let t = pts.len();
            pts.push([x, y, height]);
            lines.push_cell(&[b as i64, t as i64]);
        }
    }
    // Floor platforms
    for f in 0..=nf {
        let z = floor_h * f as f64;
        let fb = pts.len();
        pts.push([-hw, -hd, z]);
        pts.push([hw, -hd, z]);
        pts.push([hw, hd, z]);
        pts.push([-hw, hd, z]);
        polys.push_cell(&[fb as i64, (fb + 1) as i64, (fb + 2) as i64, (fb + 3) as i64]);
    }
    // Front wall
    let wb = pts.len();
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, -hd, height]);
    pts.push([-hw, -hd, height]);
    polys.push_cell(&[wb as i64, (wb + 1) as i64, (wb + 2) as i64, (wb + 3) as i64]);
    // Back wall (with gap for entry)
    let bwb = pts.len();
    pts.push([-hw, hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([hw, hd, height]);
    pts.push([-hw, hd, height]);
    polys.push_cell(&[
        bwb as i64,
        (bwb + 1) as i64,
        (bwb + 2) as i64,
        (bwb + 3) as i64,
    ]);
    // Drawbridge/ramp at top
    let ramp_b = pts.len();
    pts.push([-hw * 0.8, -hd, height]);
    pts.push([hw * 0.8, -hd, height]);
    pts.push([hw * 0.8, -hd - width, height - floor_h * 0.5]);
    pts.push([-hw * 0.8, -hd - width, height - floor_h * 0.5]);
    polys.push_cell(&[
        ramp_b as i64,
        (ramp_b + 1) as i64,
        (ramp_b + 2) as i64,
        (ramp_b + 3) as i64,
    ]);
    // Wheels
    for &x in &[-hw, hw] {
        let wc = pts.len();
        pts.push([x, 0.0, width * 0.15]);
        let na = 8;
        let rb = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            let wp = pts.len();
            pts.push([
                x,
                width * 0.15 * a.cos(),
                width * 0.15 + width * 0.15 * a.sin(),
            ]);
            lines.push_cell(&[wc as i64, wp as i64]);
        }
        for j in 0..na {
            lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
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
    fn test_siege() {
        let m = siege_tower(3.0, 8.0, 3);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 5);
    }
}
