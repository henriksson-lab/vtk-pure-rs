//! Brilliant-cut diamond gemstone shape.
use crate::data::{CellArray, Points, PolyData};

pub fn diamond_gem(radius: f64, crown_height: f64, pavilion_depth: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Table (top flat face)
    let table_r = radius * 0.55;
    let tc = pts.len();
    pts.push([0.0, 0.0, crown_height]);
    let tb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([table_r * a.cos(), table_r * a.sin(), crown_height]);
    }
    for j in 0..na {
        polys.push_cell(&[tc as i64, (tb + j) as i64, (tb + (j + 1) % na) as i64]);
    }
    // Crown (table to girdle)
    let girdle = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([radius * a.cos(), radius * a.sin(), 0.0]);
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[(tb + j) as i64, (girdle + j) as i64, (girdle + j1) as i64]);
        polys.push_cell(&[(tb + j) as i64, (girdle + j1) as i64, (tb + j1) as i64]);
    }
    // Pavilion (girdle to culet/point)
    let culet = pts.len();
    pts.push([0.0, 0.0, -pavilion_depth]);
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[(girdle + j) as i64, culet as i64, (girdle + j1) as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_diamond() {
        let m = diamond_gem(1.0, 0.3, 0.6, 16);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 30);
    }
}
