//! Collapse nearly-flat triangle pairs into quads.
use crate::data::{CellArray, PolyData};
pub fn merge_coplanar_tris(mesh: &PolyData, angle_threshold: f64) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let cos_t = angle_threshold.to_radians().cos();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut merged = vec![false; nc];
    let mut new_polys = CellArray::new();
    for (&(ea, eb), faces) in &ef {
        if faces.len() != 2 || merged[faces[0]] || merged[faces[1]] {
            continue;
        }
        if cells[faces[0]].len() != 3 || cells[faces[1]].len() != 3 {
            continue;
        }
        let n0 = fnorm(&cells[faces[0]], mesh);
        let n1 = fnorm(&cells[faces[1]], mesh);
        let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
        if dot > cos_t {
            let va: usize = cells[faces[0]]
                .iter()
                .find(|&&v| v as usize != ea && v as usize != eb)
                .map(|&v| v as usize)
                .unwrap();
            let vb: usize = cells[faces[1]]
                .iter()
                .find(|&&v| v as usize != ea && v as usize != eb)
                .map(|&v| v as usize)
                .unwrap();
            new_polys.push_cell(&[va as i64, ea as i64, vb as i64, eb as i64]);
            merged[faces[0]] = true;
            merged[faces[1]] = true;
        }
    }
    for (ci, cell) in cells.iter().enumerate() {
        if !merged[ci] {
            new_polys.push_cell(cell);
        }
    }
    let mut r = mesh.clone();
    r.polys = new_polys;
    r
}
fn fnorm(cell: &[i64], mesh: &PolyData) -> [f64; 3] {
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / l, n[1] / l, n[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_merge() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let r = merge_coplanar_tris(&m, 5.0);
        assert_eq!(r.polys.num_cells(), 1);
    } // merged to 1 quad
    #[test]
    fn test_no_merge() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = merge_coplanar_tris(&m, 5.0);
        assert_eq!(r.polys.num_cells(), 2);
    } // not coplanar
}
