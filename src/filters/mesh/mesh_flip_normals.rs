//! Reverse the winding order of all faces (flip normals).
use crate::data::{CellArray, PolyData};

pub fn flip_normals(mesh: &PolyData) -> PolyData {
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let rev: Vec<i64> = cell.iter().rev().copied().collect();
        polys.push_cell(&rev);
    }
    let mut result = mesh.clone();
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = flip_normals(&mesh);
        let cell: Vec<i64> = r.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![2, 1, 0]);
    }
}
