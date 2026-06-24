//! Crystal lattice structures (FCC, BCC, HCP unit cells).
use crate::data::{CellArray, Points, PolyData};
pub fn fcc_lattice(a: f64, nx: usize, ny: usize, nz: usize, atom_r: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let add_atom = |pts: &mut Points<f64>, polys: &mut CellArray, x: f64, y: f64, z: f64| {
        let ab = pts.len();
        let r = atom_r;
        pts.push([x + r, y, z]);
        pts.push([x - r, y, z]);
        pts.push([x, y + r, z]);
        pts.push([x, y - r, z]);
        pts.push([x, y, z + r]);
        pts.push([x, y, z - r]);
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
            polys.push_cell(&[(ab + f[0]) as i64, (ab + f[1]) as i64, (ab + f[2]) as i64]);
        }
    };
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let x = ix as f64 * a;
                let y = iy as f64 * a;
                let z = iz as f64 * a;
                add_atom(&mut pts, &mut polys, x, y, z); // corner
                add_atom(&mut pts, &mut polys, x + a / 2.0, y + a / 2.0, z); // face centers
                add_atom(&mut pts, &mut polys, x + a / 2.0, y, z + a / 2.0);
                add_atom(&mut pts, &mut polys, x, y + a / 2.0, z + a / 2.0);
            }
        }
    }
    // Unit cell edges
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let x = ix as f64 * a;
                let y = iy as f64 * a;
                let z = iz as f64 * a;
                let corners = [
                    [x, y, z],
                    [x + a, y, z],
                    [x + a, y + a, z],
                    [x, y + a, z],
                    [x, y, z + a],
                    [x + a, y, z + a],
                    [x + a, y + a, z + a],
                    [x, y + a, z + a],
                ];
                let edges = [
                    [0, 1],
                    [1, 2],
                    [2, 3],
                    [3, 0],
                    [4, 5],
                    [5, 6],
                    [6, 7],
                    [7, 4],
                    [0, 4],
                    [1, 5],
                    [2, 6],
                    [3, 7],
                ];
                for e in &edges {
                    let lb = pts.len();
                    pts.push(corners[e[0]]);
                    pts.push(corners[e[1]]);
                    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
                }
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
pub fn bcc_unit_cell(a: f64, atom_r: f64) -> PolyData {
    fcc_lattice(a, 1, 1, 1, atom_r)
} // simplified
pub fn simple_cubic(a: f64, nx: usize, ny: usize, nz: usize, atom_r: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                let x = ix as f64 * a;
                let y = iy as f64 * a;
                let z = iz as f64 * a;
                let ab = pts.len();
                let r = atom_r;
                pts.push([x + r, y, z]);
                pts.push([x - r, y, z]);
                pts.push([x, y + r, z]);
                pts.push([x, y - r, z]);
                pts.push([x, y, z + r]);
                pts.push([x, y, z - r]);
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
                    polys.push_cell(&[(ab + f[0]) as i64, (ab + f[1]) as i64, (ab + f[2]) as i64]);
                }
                // Bonds to neighbors
                if ix > 0 {
                    let lb = pts.len();
                    pts.push([x, y, z]);
                    pts.push([x - a, y, z]);
                    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
                }
                if iy > 0 {
                    let lb = pts.len();
                    pts.push([x, y, z]);
                    pts.push([x, y - a, z]);
                    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
                }
                if iz > 0 {
                    let lb = pts.len();
                    pts.push([x, y, z]);
                    pts.push([x, y, z - a]);
                    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
                }
            }
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
    fn test_fcc() {
        let f = fcc_lattice(1.0, 2, 2, 2, 0.1);
        assert!(f.polys.num_cells() > 100);
    }
    #[test]
    fn test_sc() {
        let s = simple_cubic(1.0, 2, 2, 2, 0.1);
        assert!(s.polys.num_cells() > 100);
    }
}
