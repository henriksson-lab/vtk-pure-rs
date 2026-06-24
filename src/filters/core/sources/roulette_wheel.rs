//! Roulette wheel with numbered pockets.
use crate::data::{CellArray, Points, PolyData};

pub fn roulette_wheel(radius: f64, n_pockets: usize, na: usize) -> PolyData {
    let np = n_pockets.max(10);
    let na = na.max(np * 2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let inner_r = radius * 0.3;
    let pocket_r = radius * 0.85;
    // Center cone
    let cc = pts.len();
    pts.push([0.0, 0.0, radius * 0.15]);
    let cb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([inner_r * a.cos(), inner_r * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[cc as i64, (cb + j) as i64, (cb + (j + 1) % na) as i64]);
    }
    // Pocket ring
    let ob = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([pocket_r * a.cos(), pocket_r * a.sin(), 0.0]);
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (cb + j) as i64,
            (ob + j * 2) as i64,
            (ob + j1 * 2) as i64,
            (cb + j1) as i64,
        ]);
        polys.push_cell(&[
            (ob + j * 2) as i64,
            (ob + j * 2 + 1) as i64,
            (ob + j1 * 2 + 1) as i64,
            (ob + j1 * 2) as i64,
        ]);
    }
    // Pocket dividers
    for p in 0..np {
        let a = 2.0 * std::f64::consts::PI * p as f64 / np as f64;
        let d0 = pts.len();
        pts.push([pocket_r * a.cos(), pocket_r * a.sin(), 0.01]);
        let d1 = pts.len();
        pts.push([radius * a.cos(), radius * a.sin(), 0.01]);
        lines.push_cell(&[d0 as i64, d1 as i64]);
    }
    // Outer rim (raised)
    let rb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([
            radius * 1.05 * a.cos(),
            radius * 1.05 * a.sin(),
            radius * 0.05,
        ]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (ob + j * 2 + 1) as i64,
            (rb + j) as i64,
            (rb + j1) as i64,
            (ob + j1 * 2 + 1) as i64,
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
    fn test_roulette() {
        let m = roulette_wheel(5.0, 37, 74);
        assert!(m.points.len() > 200);
        assert!(m.polys.num_cells() > 100);
    }
}
