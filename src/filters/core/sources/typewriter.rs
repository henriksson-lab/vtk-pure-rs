//! Typewriter (carriage, platen, keyboard outline).
use crate::data::{CellArray, Points, PolyData};

pub fn typewriter(width: f64, depth: f64, height: f64) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Base body
    let bb = pts.len();
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    pts.push([-hw, -hd, height * 0.4]);
    pts.push([hw, -hd, height * 0.4]);
    pts.push([hw, hd, height * 0.3]);
    pts.push([-hw, hd, height * 0.3]);
    for s in 0..4 {
        let i = bb + s;
        let j = bb + (s + 1) % 4;
        let k = bb + 4 + (s + 1) % 4;
        let l = bb + 4 + s;
        polys.push_cell(&[i as i64, j as i64, k as i64, l as i64]);
    }
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 7) as i64,
    ]); // top
        // Platen (cylinder at back)
    let platen_r = height * 0.08;
    let na = 10;
    let pb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([
            -hw * 0.9,
            hd * 0.5 + platen_r * a.cos(),
            height * 0.5 + platen_r * a.sin(),
        ]);
        pts.push([
            hw * 0.9,
            hd * 0.5 + platen_r * a.cos(),
            height * 0.5 + platen_r * a.sin(),
        ]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (pb + j * 2) as i64,
            (pb + j * 2 + 1) as i64,
            (pb + j1 * 2 + 1) as i64,
            (pb + j1 * 2) as i64,
        ]);
    }
    // Carriage return lever
    let cl0 = pts.len();
    pts.push([hw * 0.9, hd * 0.5, height * 0.5]);
    let cl1 = pts.len();
    pts.push([hw * 1.2, hd * 0.3, height * 0.6]);
    lines.push_cell(&[cl0 as i64, cl1 as i64]);
    // Keyboard rows (3 rows of keys as dots)
    for row in 0..3 {
        let y = -hd * 0.6 + depth * 0.15 * row as f64;
        let z = height * 0.4 - 0.02 * row as f64;
        let n_keys = 10 - row;
        for k in 0..n_keys {
            let x = -hw * 0.7 + width * 0.7 * k as f64 / (n_keys - 1).max(1) as f64;
            let kb = pts.len();
            for j in 0..6 {
                let a = 2.0 * std::f64::consts::PI * j as f64 / 6.0;
                pts.push([x + width * 0.02 * a.cos(), y, z + width * 0.02 * a.sin()]);
            }
            for j in 0..6 {
                lines.push_cell(&[(kb + j) as i64, (kb + (j + 1) % 6) as i64]);
            }
        }
    }
    // Paper (rectangle above platen)
    let ppb = pts.len();
    pts.push([-hw * 0.7, hd * 0.5, height * 0.5 + platen_r]);
    pts.push([hw * 0.7, hd * 0.5, height * 0.5 + platen_r]);
    pts.push([hw * 0.7, hd * 0.5, height]);
    pts.push([-hw * 0.7, hd * 0.5, height]);
    polys.push_cell(&[
        ppb as i64,
        (ppb + 1) as i64,
        (ppb + 2) as i64,
        (ppb + 3) as i64,
    ]);
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
    fn test_typewriter() {
        let m = typewriter(6.0, 4.0, 3.0);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 10);
        assert!(m.lines.num_cells() > 30);
    }
}
