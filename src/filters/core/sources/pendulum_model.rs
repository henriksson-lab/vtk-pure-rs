//! Pendulum geometry (bob + string + pivot).
use crate::data::{CellArray, Points, PolyData};
pub fn pendulum(
    string_length: f64,
    bob_radius: f64,
    angle_degrees: f64,
    resolution: usize,
) -> PolyData {
    let _res = resolution.max(6);
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
    let sb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([bx, 0.0, bz]);
    lines.push_cell(&[sb as i64, (sb + 1) as i64]);
    // Bob (sphere approx - octahedron)
    let bb = pts.len();
    pts.push([bx + bob_radius, 0.0, bz]);
    pts.push([bx - bob_radius, 0.0, bz]);
    pts.push([bx, bob_radius, bz]);
    pts.push([bx, -bob_radius, bz]);
    pts.push([bx, 0.0, bz + bob_radius]);
    pts.push([bx, 0.0, bz - bob_radius]);
    let faces = [
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [0, 5, 2],
        [2, 5, 1],
        [1, 5, 3],
        [3, 5, 0],
    ];
    for f in &faces {
        polys.push_cell(&[(bb + f[0]) as i64, (bb + f[1]) as i64, (bb + f[2]) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
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
        let bb = pts.len();
        pts.push([bx + bob_r, 0.0, bz]);
        pts.push([bx - bob_r, 0.0, bz]);
        pts.push([bx, bob_r, bz]);
        pts.push([bx, -bob_r, bz]);
        pts.push([bx, 0.0, bz + bob_r]);
        pts.push([bx, 0.0, bz - bob_r]);
        for f in &[
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [0, 5, 2],
            [2, 5, 1],
            [1, 5, 3],
            [3, 5, 0],
        ] {
            polys.push_cell(&[(bb + f[0]) as i64, (bb + f[1]) as i64, (bb + f[2]) as i64]);
        }
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
