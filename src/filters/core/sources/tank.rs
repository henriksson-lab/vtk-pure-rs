//! Storage tank (cylindrical with domed ends).
use crate::data::{CellArray, Points, PolyData};
pub fn vertical_tank(radius: f64, height: f64, dome_height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Cylinder
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Bottom dome (hemispherical)
    let bd = pts.len();
    pts.push([0.0, 0.0, -dome_height]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[bd as i64, j as i64, i as i64]);
    }
    // Top dome
    let td = pts.len();
    pts.push([0.0, 0.0, height + dome_height]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[td as i64, (res + i) as i64, (res + j) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn horizontal_tank(radius: f64, length: f64, dome_height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(6);
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for ring in 0..=1 {
        let x = if ring == 0 { -hl } else { hl };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, radius * a.cos(), radius * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    let ld = pts.len();
    pts.push([-hl - dome_height, 0.0, 0.0]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[ld as i64, j as i64, i as i64]);
    }
    let rd = pts.len();
    pts.push([hl + dome_height, 0.0, 0.0]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[rd as i64, (res + i) as i64, (res + j) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_v() {
        let t = vertical_tank(2.0, 5.0, 0.5, 12);
        assert!(t.polys.num_cells() > 20);
    }
    #[test]
    fn test_h() {
        let t = horizontal_tank(1.5, 6.0, 0.4, 10);
        assert!(t.polys.num_cells() > 15);
    }
}
