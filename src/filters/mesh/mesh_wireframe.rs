//! Extract wireframe (all edges as lines) from a mesh.
use crate::data::{CellArray, PolyData};

pub fn wireframe(mesh: &PolyData) -> PolyData {
    let mut edges = std::collections::HashSet::new();
    let mut lines = CellArray::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            let e = if a < b { (a, b) } else { (b, a) };
            if edges.insert(e) {
                lines.push_cell(&[a, b]);
            }
        }
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.lines = lines;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wireframe() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = wireframe(&mesh);
        assert_eq!(r.lines.num_cells(), 3);
    }
}
