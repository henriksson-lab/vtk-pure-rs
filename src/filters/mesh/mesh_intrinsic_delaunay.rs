//! Intrinsic Delaunay triangulation (edge flipping to maximize min angle).
use crate::data::{CellArray, PolyData};
pub fn intrinsic_delaunay(mesh: &PolyData, max_iterations: usize) -> PolyData {
    let mut cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    for _ in 0..max_iterations {
        let mut flipped = false;
        let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
            std::collections::HashMap::new();
        for (ci, c) in cells.iter().enumerate() {
            let n = c.len();
            for i in 0..n {
                let a = c[i] as usize;
                let b = c[(i + 1) % n] as usize;
                ef.entry((a.min(b), a.max(b))).or_default().push(ci);
            }
        }
        for (&(ea, eb), faces) in &ef {
            if faces.len() != 2 {
                continue;
            }
            if cells[faces[0]].len() != 3 || cells[faces[1]].len() != 3 {
                continue;
            }
            let va = cells[faces[0]]
                .iter()
                .find(|&&v| v as usize != ea && v as usize != eb)
                .map(|&v| v as usize);
            let vb = cells[faces[1]]
                .iter()
                .find(|&&v| v as usize != ea && v as usize != eb)
                .map(|&v| v as usize);
            if let (Some(va), Some(vb)) = (va, vb) {
                let old_min = min_angle_pair(&cells[faces[0]], &cells[faces[1]], mesh);
                let new_t1 = vec![va as i64, ea as i64, vb as i64];
                let new_t2 = vec![va as i64, vb as i64, eb as i64];
                let new_min = min_angle_pair(&new_t1, &new_t2, mesh);
                if new_min > old_min + 0.01 {
                    cells[faces[0]] = new_t1;
                    cells[faces[1]] = new_t2;
                    flipped = true;
                }
            }
        }
        if !flipped {
            break;
        }
    }
    let mut polys = CellArray::new();
    for c in &cells {
        polys.push_cell(c);
    }
    let mut r = mesh.clone();
    r.polys = polys;
    r
}
fn min_angle_pair(a: &[i64], b: &[i64], mesh: &PolyData) -> f64 {
    let mut mn = 180.0f64;
    for cell in [a, b] {
        if cell.len() != 3 {
            continue;
        }
        let p = [
            mesh.points.get(cell[0] as usize),
            mesh.points.get(cell[1] as usize),
            mesh.points.get(cell[2] as usize),
        ];
        for i in 0..3 {
            let v1 = [
                p[(i + 1) % 3][0] - p[i][0],
                p[(i + 1) % 3][1] - p[i][1],
                p[(i + 1) % 3][2] - p[i][2],
            ];
            let v2 = [
                p[(i + 2) % 3][0] - p[i][0],
                p[(i + 2) % 3][1] - p[i][1],
                p[(i + 2) % 3][2] - p[i][2],
            ];
            let d = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
            let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
            let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
            if l1 > 1e-15 && l2 > 1e-15 {
                mn = mn.min((d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees());
            }
        }
    }
    mn
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let r = intrinsic_delaunay(&m, 10);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
