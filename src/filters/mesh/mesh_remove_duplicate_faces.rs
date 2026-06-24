//! Remove duplicate faces (faces with same vertex set regardless of order).
use crate::data::{CellArray, PolyData};

pub fn remove_duplicate_faces(mesh: &PolyData) -> PolyData {
    let mut seen = std::collections::HashSet::new();
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let mut sorted: Vec<i64> = cell.to_vec();
        sorted.sort();
        let key: Vec<i64> = sorted;
        if seen.insert(key) {
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
    fn test_dedup() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2], [0, 1, 2], [1, 0, 2]], // three copies (two duplicates)
        );
        let r = remove_duplicate_faces(&mesh);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
