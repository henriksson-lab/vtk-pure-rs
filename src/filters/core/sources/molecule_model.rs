//! Simple ball-and-stick molecular model.
use crate::data::{CellArray, Points, PolyData};
pub fn molecule(
    atoms: &[[f64; 3]],
    radii: &[f64],
    bonds: &[(usize, usize)],
    resolution: usize,
) -> PolyData {
    let _res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Atoms (octahedra as sphere approximation)
    for (ai, &pos) in atoms.iter().enumerate() {
        let r = if ai < radii.len() { radii[ai] } else { 0.3 };
        let ab = pts.len();
        pts.push([pos[0] + r, pos[1], pos[2]]);
        pts.push([pos[0] - r, pos[1], pos[2]]);
        pts.push([pos[0], pos[1] + r, pos[2]]);
        pts.push([pos[0], pos[1] - r, pos[2]]);
        pts.push([pos[0], pos[1], pos[2] + r]);
        pts.push([pos[0], pos[1], pos[2] - r]);
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
            polys.push_cell(&[(ab + f[0]) as i64, (ab + f[1]) as i64, (ab + f[2]) as i64]);
        }
    }
    // Bonds (lines)
    for &(a, b) in bonds {
        if a < atoms.len() && b < atoms.len() {
            let lb = pts.len();
            pts.push(atoms[a]);
            pts.push(atoms[b]);
            lines.push_cell(&[lb as i64, (lb + 1) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
pub fn water_molecule() -> PolyData {
    let atoms = [[0.0, 0.0, 0.0], [0.757, 0.586, 0.0], [-0.757, 0.586, 0.0]];
    let radii = [0.4, 0.25, 0.25]; // O, H, H
    molecule(&atoms, &radii, &[(0, 1), (0, 2)], 6)
}
pub fn methane_molecule() -> PolyData {
    let t = 1.0 / 3.0f64.sqrt();
    let atoms = [
        [0.0, 0.0, 0.0],
        [t, t, t],
        [t, -t, -t],
        [-t, t, -t],
        [-t, -t, t],
    ];
    let radii = [0.4, 0.25, 0.25, 0.25, 0.25];
    molecule(&atoms, &radii, &[(0, 1), (0, 2), (0, 3), (0, 4)], 6)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_water() {
        let w = water_molecule();
        assert!(w.polys.num_cells() >= 24);
        assert_eq!(w.lines.num_cells(), 2);
    }
    #[test]
    fn test_methane() {
        let m = methane_molecule();
        assert!(m.polys.num_cells() >= 40);
        assert_eq!(m.lines.num_cells(), 4);
    }
    #[test]
    fn test_custom() {
        let m = molecule(
            &[[0.0, 0.0, 0.0], [1.5, 0.0, 0.0]],
            &[0.3, 0.3],
            &[(0, 1)],
            8,
        );
        assert!(m.polys.num_cells() > 10);
        assert_eq!(m.lines.num_cells(), 1);
    }
}
