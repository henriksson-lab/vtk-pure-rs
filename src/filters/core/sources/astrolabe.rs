//! Astrolabe (medieval astronomical instrument).
use crate::data::{CellArray, Points, PolyData};

pub fn astrolabe(radius: f64, na: usize) -> PolyData {
    let na = na.max(24);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Mater (main disk)
    let mc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[
            mc as i64,
            (mc + 1 + j) as i64,
            (mc + 1 + (j + 1) % na) as i64,
        ]);
    }
    // Outer rim
    let rim_r = radius * 1.05;
    let rb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([rim_r * a.cos(), rim_r * a.sin(), 0.01]);
    }
    for j in 0..na {
        lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
    }
    // Degree markings (every 30 degrees = zodiac signs)
    for i in 0..12 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 12.0;
        let m0 = pts.len();
        pts.push([radius * 0.9 * a.cos(), radius * 0.9 * a.sin(), 0.01]);
        let m1 = pts.len();
        pts.push([rim_r * a.cos(), rim_r * a.sin(), 0.01]);
        lines.push_cell(&[m0 as i64, m1 as i64]);
    }
    // Rete (star pointer overlay) - simplified as a few radial lines and circles
    let rete_r = radius * 0.7;
    let ret_b = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([rete_r * a.cos(), rete_r * a.sin(), 0.02]);
    }
    for j in 0..na {
        lines.push_cell(&[(ret_b + j) as i64, (ret_b + (j + 1) % na) as i64]);
    }
    // Alidade (rule/pointer)
    let al0 = pts.len();
    pts.push([-radius * 0.95, 0.0, 0.03]);
    let al1 = pts.len();
    pts.push([radius * 0.95, 0.0, 0.03]);
    lines.push_cell(&[al0 as i64, al1 as i64]);
    // Throne (suspension ring at top)
    let throne_na = 8;
    let throne_r = radius * 0.12;
    let throne_cy = radius * 1.15;
    let tb = pts.len();
    for j in 0..throne_na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / throne_na as f64;
        pts.push([throne_r * a.cos(), throne_cy + throne_r * a.sin(), 0.0]);
    }
    for j in 0..throne_na {
        lines.push_cell(&[(tb + j) as i64, (tb + (j + 1) % throne_na) as i64]);
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
    fn test_astrolabe() {
        let m = astrolabe(5.0, 24);
        assert!(m.points.len() > 70);
        assert!(m.polys.num_cells() > 20);
        assert!(m.lines.num_cells() > 30);
    }
}
