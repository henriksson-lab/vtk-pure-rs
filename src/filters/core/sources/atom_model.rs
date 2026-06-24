//! Bohr atom model (nucleus + electron orbits).
use crate::data::{CellArray, Points, PolyData};
pub fn bohr_atom(
    nucleus_r: f64,
    num_shells: usize,
    electrons_per_shell: &[usize],
    resolution: usize,
) -> PolyData {
    let res = resolution.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Nucleus (octahedron)
    let nb = pts.len();
    let nr = nucleus_r;
    pts.push([nr, 0.0, 0.0]);
    pts.push([-nr, 0.0, 0.0]);
    pts.push([0.0, nr, 0.0]);
    pts.push([0.0, -nr, 0.0]);
    pts.push([0.0, 0.0, nr]);
    pts.push([0.0, 0.0, -nr]);
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
        polys.push_cell(&[(nb + f[0]) as i64, (nb + f[1]) as i64, (nb + f[2]) as i64]);
    }
    // Electron shells (circles) and electrons (small spheres)
    let ns = num_shells.min(electrons_per_shell.len());
    for si in 0..ns {
        let shell_r = nucleus_r * (2.0 + si as f64 * 1.5);
        let ne = electrons_per_shell[si];
        // Orbit ring
        let mut orbit_ids = Vec::new();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            let idx = pts.len();
            pts.push([shell_r * a.cos(), shell_r * a.sin(), 0.0]);
            orbit_ids.push(idx as i64);
        }
        orbit_ids.push(orbit_ids[0]);
        lines.push_cell(&orbit_ids);
        // Electrons
        let er = nucleus_r * 0.3;
        for ei in 0..ne {
            let a = 2.0 * std::f64::consts::PI * ei as f64 / ne as f64;
            let ex = shell_r * a.cos();
            let ey = shell_r * a.sin();
            let eb = pts.len();
            pts.push([ex + er, ey, 0.0]);
            pts.push([ex - er, ey, 0.0]);
            pts.push([ex, ey + er, 0.0]);
            pts.push([ex, ey - er, 0.0]);
            pts.push([ex, ey, er]);
            pts.push([ex, ey, -er]);
            for f in &faces {
                polys.push_cell(&[(eb + f[0]) as i64, (eb + f[1]) as i64, (eb + f[2]) as i64]);
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
pub fn hydrogen_atom() -> PolyData {
    bohr_atom(0.3, 1, &[1], 16)
}
pub fn helium_atom() -> PolyData {
    bohr_atom(0.3, 1, &[2], 16)
}
pub fn carbon_atom() -> PolyData {
    bohr_atom(0.4, 2, &[2, 4], 16)
}
pub fn oxygen_atom() -> PolyData {
    bohr_atom(0.4, 2, &[2, 6], 16)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_h() {
        let h = hydrogen_atom();
        assert!(h.polys.num_cells() >= 16);
        assert!(h.lines.num_cells() >= 1);
    }
    #[test]
    fn test_c() {
        let c = carbon_atom();
        assert!(c.polys.num_cells() > 40);
        assert!(c.lines.num_cells() >= 2);
    }
    #[test]
    fn test_custom() {
        let a = bohr_atom(0.5, 3, &[2, 8, 1], 12);
        assert!(a.polys.num_cells() > 50);
    }
}
