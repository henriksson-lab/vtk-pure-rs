//! Skateboard/BMX ramp (quarter pipe, half pipe) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn quarter_pipe(radius: f64, width: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let a = t * std::f64::consts::FRAC_PI_2;
        let x = radius * (1.0 - a.cos());
        let z = radius * a.sin();
        pts.push([x, -hw, z]);
        pts.push([x, hw, z]);
    }
    for i in 0..res {
        let b = i * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
    }
    // Flat bottom
    let b = pts.len();
    pts.push([0.0, -hw, 0.0]);
    pts.push([0.0, hw, 0.0]);
    pts.push([-radius * 0.5, -hw, 0.0]);
    pts.push([-radius * 0.5, hw, 0.0]);
    polys.push_cell(&[b as i64, (b + 1) as i64, (b + 3) as i64, (b + 2) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn half_pipe(radius: f64, width: f64, flat_length: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let hw = width / 2.0;
    let hl = flat_length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Left quarter
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let a = std::f64::consts::PI + t * std::f64::consts::FRAC_PI_2;
        let x = -hl + radius * a.cos();
        let z = radius + radius * a.sin();
        pts.push([x, -hw, z]);
        pts.push([x, hw, z]);
    }
    // Flat
    let fb = pts.len();
    pts.push([-hl, -hw, 0.0]);
    pts.push([-hl, hw, 0.0]);
    pts.push([hl, -hw, 0.0]);
    pts.push([hl, hw, 0.0]);
    polys.push_cell(&[fb as i64, (fb + 2) as i64, (fb + 3) as i64, (fb + 1) as i64]);
    // Right quarter
    let rb = pts.len();
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let a = std::f64::consts::PI * 1.5 + t * std::f64::consts::FRAC_PI_2;
        let x = hl + radius * a.cos();
        let z = radius + radius * a.sin();
        pts.push([x, -hw, z]);
        pts.push([x, hw, z]);
    }
    // Connect curves
    for base in [0, rb] {
        for i in 0..res {
            let b = base + i * 2;
            polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
        }
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
    fn test_quarter() {
        let q = quarter_pipe(2.0, 3.0, 8);
        assert!(q.polys.num_cells() >= 8);
    }
    #[test]
    fn test_half() {
        let h = half_pipe(2.0, 3.0, 4.0, 8);
        assert!(h.polys.num_cells() > 10);
        assert!((h.points[0][0] + 4.0).abs() < 1e-12);
        assert!((h.points[0][2] - 2.0).abs() < 1e-12);
        assert!((h.points[16][0] + 2.0).abs() < 1e-12);
        assert!(h.points[16][2].abs() < 1e-12);
        assert!((h.points[22][0] - 2.0).abs() < 1e-12);
        assert!(h.points[22][2].abs() < 1e-12);
        assert!((h.points[38][0] - 4.0).abs() < 1e-12);
        assert!((h.points[38][2] - 2.0).abs() < 1e-12);
    }
}
