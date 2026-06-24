//! Remove degenerate faces (zero area or collapsed edges).
use crate::data::{CellArray, PolyData};

pub fn remove_degenerate_faces(mesh: &PolyData, min_area: f64) -> PolyData {
    let n = mesh.points.len();
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        // Check for repeated vertices
        let unique: std::collections::HashSet<i64> = cell.iter().copied().collect();
        if unique.len() < 3 {
            continue;
        }
        // Check area
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let cross = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if area >= min_area {
            polys.push_cell(&cell.to_vec());
        }
    }
    let mut result = mesh.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_degenerate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ], // last is degenerate
            vec![[0, 1, 2], [0, 1, 3]], // second face has zero area (0 and 3 are same point)
        );
        let r = remove_degenerate_faces(&mesh, 1e-10);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
