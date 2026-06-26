//! Simple ball-and-stick molecular model.
use crate::data::{CellArray, Points, PolyData};

fn add_sphere(
    pts: &mut Points<f64>,
    polys: &mut CellArray,
    center: [f64; 3],
    radius: f64,
    resolution: usize,
) -> usize {
    let base = pts.len();
    let nr = (resolution / 2).max(3);
    pts.push([center[0], center[1], center[2] + radius]);
    for ring in 1..nr {
        let phi = std::f64::consts::PI * ring as f64 / nr as f64;
        for j in 0..resolution {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / resolution as f64;
            pts.push([
                center[0] + radius * phi.sin() * theta.cos(),
                center[1] + radius * phi.sin() * theta.sin(),
                center[2] + radius * phi.cos(),
            ]);
        }
    }
    pts.push([center[0], center[1], center[2] - radius]);
    let south = pts.len() - 1;

    for j in 0..resolution {
        polys.push_cell(&[
            base as i64,
            (base + 1 + j) as i64,
            (base + 1 + (j + 1) % resolution) as i64,
        ]);
    }
    for ring in 0..(nr - 2) {
        let r0 = base + 1 + ring * resolution;
        let r1 = base + 1 + (ring + 1) * resolution;
        for j in 0..resolution {
            let j1 = (j + 1) % resolution;
            polys.push_cell(&[(r0 + j) as i64, (r1 + j) as i64, (r1 + j1) as i64]);
            polys.push_cell(&[(r0 + j) as i64, (r1 + j1) as i64, (r0 + j1) as i64]);
        }
    }
    let last_ring = base + 1 + (nr - 2) * resolution;
    for j in 0..resolution {
        polys.push_cell(&[
            (last_ring + j) as i64,
            south as i64,
            (last_ring + (j + 1) % resolution) as i64,
        ]);
    }
    base
}

pub fn molecule(
    atoms: &[[f64; 3]],
    radii: &[f64],
    bonds: &[(usize, usize)],
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    for (ai, &pos) in atoms.iter().enumerate() {
        let r = if ai < radii.len() { radii[ai] } else { 0.3 };
        add_sphere(&mut pts, &mut polys, pos, r, res);
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
