//! Pendulum geometry (bob + string + pivot).
use crate::data::{CellArray, Points, PolyData};
pub fn pendulum(
    string_length: f64,
    bob_radius: f64,
    angle_degrees: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let angle = angle_degrees.to_radians();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pivot
    pts.push([0.0, 0.0, 0.0]);
    // Bob position
    let bx = string_length * angle.sin();
    let bz = -string_length * angle.cos();
    // String
    let bob_center = pts.len();
    pts.push([bx, 0.0, bz]);
    lines.push_cell(&[0, bob_center as i64]);
    // Bob
    add_bob(&mut pts, &mut polys, bx, bz, bob_radius, res);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}

fn add_bob(
    pts: &mut Points<f64>,
    polys: &mut CellArray,
    bx: f64,
    bz: f64,
    bob_radius: f64,
    resolution: usize,
) {
    let np = 3;
    let top = pts.len();
    pts.push([bx, 0.0, bz + bob_radius]);
    for p in 1..np {
        let phi = std::f64::consts::PI * p as f64 / np as f64;
        for j in 0..resolution {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / resolution as f64;
            pts.push([
                bx + bob_radius * phi.sin() * theta.cos(),
                bob_radius * phi.sin() * theta.sin(),
                bz + bob_radius * phi.cos(),
            ]);
        }
    }
    let bottom = pts.len();
    pts.push([bx, 0.0, bz - bob_radius]);

    for j in 0..resolution {
        polys.push_cell(&[
            top as i64,
            (top + 1 + j) as i64,
            (top + 1 + (j + 1) % resolution) as i64,
        ]);
    }
    for p in 0..(np - 2) {
        let b0 = top + 1 + p * resolution;
        let b1 = top + 1 + (p + 1) * resolution;
        for j in 0..resolution {
            let j1 = (j + 1) % resolution;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    let bottom_ring = top + 1 + (np - 2) * resolution;
    for j in 0..resolution {
        let j1 = (j + 1) % resolution;
        polys.push_cell(&[
            (bottom_ring + j) as i64,
            (bottom_ring + j1) as i64,
            bottom as i64,
        ]);
    }
}
pub fn double_pendulum(l1: f64, l2: f64, bob_r: f64, a1: f64, a2: f64) -> PolyData {
    let a1r = a1.to_radians();
    let a2r = a2.to_radians();
    let x1 = l1 * a1r.sin();
    let z1 = -l1 * a1r.cos();
    let x2 = x1 + l2 * (a1r + a2r).sin();
    let z2 = z1 - l2 * (a1r + a2r).cos();
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([x1, 0.0, z1]);
    pts.push([x2, 0.0, z2]);
    lines.push_cell(&[0, 1]);
    lines.push_cell(&[1, 2]);
    // Bobs
    for &(bx, bz) in &[(x1, z1), (x2, z2)] {
        add_bob(&mut pts, &mut polys, bx, bz, bob_r, 8);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single() {
        let p = pendulum(3.0, 0.2, 30.0, 8);
        assert!(p.polys.num_cells() >= 8);
        assert!(p.lines.num_cells() >= 1);
    }
    #[test]
    fn test_double() {
        let p = double_pendulum(2.0, 1.5, 0.15, 30.0, 45.0);
        assert!(p.polys.num_cells() >= 16);
        assert_eq!(p.lines.num_cells(), 2);
    }
}
