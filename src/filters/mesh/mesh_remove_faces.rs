//! Remove faces by index or predicate.

use crate::data::{CellArray, Points, PolyData};

/// Remove faces at given indices.
pub fn remove_faces_by_index(
    mesh: &PolyData,
    indices: &std::collections::HashSet<usize>,
) -> PolyData {
    remove_faces_if(mesh, |i, _| indices.contains(&i))
}

/// Remove faces matching a predicate.
pub fn remove_faces_if(mesh: &PolyData, pred: impl Fn(usize, &[i64]) -> bool) -> PolyData {
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if !pred(ci, cell) {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    let mut pt_map = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept {
        polys.push_cell(
            &cell
                .iter()
                .map(|&v| pt_map[v as usize] as i64)
                .collect::<Vec<_>>(),
        );
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}

/// Remove small faces (area below threshold).
pub fn remove_small_faces(mesh: &PolyData, min_area: f64) -> PolyData {
    remove_faces_if(mesh, |_, cell| {
        if cell.len() < 3 {
            return true;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        0.5 * (cx * cx + cy * cy + cz * cz).sqrt() < min_area
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_index() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        let mut s = std::collections::HashSet::new();
        s.insert(0);
        let r = remove_faces_by_index(&mesh, &s);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_remove_small() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, 10.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.01, 0.0, 0.0],
                [0.0, 0.01, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = remove_small_faces(&mesh, 0.01);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
