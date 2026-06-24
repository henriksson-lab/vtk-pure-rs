//! Body-Centered Cubic (BCC) crystal lattice.
use crate::data::{CellArray, Points, PolyData};

pub fn crystal_bcc(a: f64, nx: usize, ny: usize, nz: usize) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut atom_indices: std::collections::HashMap<(i32, i32, i32), usize> =
        std::collections::HashMap::new();
    // Corner atoms
    for ix in 0..=nx {
        for iy in 0..=ny {
            for iz in 0..=nz {
                let idx = pts.len();
                pts.push([a * ix as f64, a * iy as f64, a * iz as f64]);
                atom_indices.insert((ix as i32 * 2, iy as i32 * 2, iz as i32 * 2), idx);
            }
        }
    }
    // Body center atoms
    for ix in 0..nx {
        for iy in 0..ny {
            for iz in 0..nz {
                let idx = pts.len();
                pts.push([
                    a * (ix as f64 + 0.5),
                    a * (iy as f64 + 0.5),
                    a * (iz as f64 + 0.5),
                ]);
                atom_indices.insert(
                    (ix as i32 * 2 + 1, iy as i32 * 2 + 1, iz as i32 * 2 + 1),
                    idx,
                );
            }
        }
    }
    // Bonds: each body center connects to 8 corners
    for ix in 0..nx {
        for iy in 0..ny {
            for iz in 0..nz {
                let center_key = (ix as i32 * 2 + 1, iy as i32 * 2 + 1, iz as i32 * 2 + 1);
                if let Some(&ci) = atom_indices.get(&center_key) {
                    for &(dx, dy, dz) in &[
                        (0, 0, 0),
                        (1, 0, 0),
                        (0, 1, 0),
                        (0, 0, 1),
                        (1, 1, 0),
                        (1, 0, 1),
                        (0, 1, 1),
                        (1, 1, 1),
                    ] {
                        let corner_key = (
                            (ix + dx) as i32 * 2,
                            (iy + dy) as i32 * 2,
                            (iz + dz) as i32 * 2,
                        );
                        if let Some(&cj) = atom_indices.get(&corner_key) {
                            lines.push_cell(&[ci as i64, cj as i64]);
                        }
                    }
                }
            }
        }
    }
    // Vertex markers
    let mut verts = CellArray::new();
    for i in 0..pts.len() {
        verts.push_cell(&[i as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m.verts = verts;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bcc() {
        let m = crystal_bcc(3.0, 2, 2, 2);
        assert!(m.points.len() > 20);
        assert!(m.lines.num_cells() > 30);
    }
}
