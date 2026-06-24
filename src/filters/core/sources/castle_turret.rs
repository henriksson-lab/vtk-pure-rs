//! Castle turret with crenellations.
use crate::data::{CellArray, Points, PolyData};

pub fn castle_turret(radius: f64, height: f64, n_merlons: usize, n_angular: usize) -> PolyData {
    let na = n_angular.max(8);
    let nm = n_merlons.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main cylinder
    for f in 0..=1 {
        let z = height * f as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([radius * a.cos(), radius * a.sin(), z]);
        }
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[j as i64, (na + j) as i64, (na + j1) as i64]);
        polys.push_cell(&[j as i64, (na + j1) as i64, j1 as i64]);
    }
    // Crenellations (merlons)
    let merlon_h = height * 0.2;
    let merlon_w = 2.0 * std::f64::consts::PI / (nm as f64 * 2.0);
    for m in 0..nm {
        let center_angle = 2.0 * std::f64::consts::PI * m as f64 / nm as f64;
        let a0 = center_angle - merlon_w / 2.0;
        let a1 = center_angle + merlon_w / 2.0;
        let b = pts.len();
        pts.push([radius * a0.cos(), radius * a0.sin(), height]);
        pts.push([radius * a1.cos(), radius * a1.sin(), height]);
        pts.push([radius * a1.cos(), radius * a1.sin(), height + merlon_h]);
        pts.push([radius * a0.cos(), radius * a0.sin(), height + merlon_h]);
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
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
    fn test_turret() {
        let m = castle_turret(2.0, 5.0, 6, 16);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 10);
    }
}
