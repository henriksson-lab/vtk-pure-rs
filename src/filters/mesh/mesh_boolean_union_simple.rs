//! Simple boolean union by appending two meshes.
use crate::data::PolyData;
pub fn boolean_union_append(a: &PolyData, b: &PolyData) -> PolyData {
    super::merge_ops::append_meshes(&[a, b])
}
pub fn boolean_xor_simple(a: &PolyData, b: &PolyData) -> PolyData {
    // Keep faces of A outside B and faces of B outside A
    let a_outside_b = super::mesh_boolean_simple::extract_outside(a, b);
    let b_outside_a = super::mesh_boolean_simple::extract_outside(b, a);
    super::merge_ops::append_meshes(&[&a_outside_b, &b_outside_a])
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_union() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_union_append(&a, &b);
        assert_eq!(r.polys.num_cells(), 2);
        assert_eq!(r.points.len(), 6);
    }
    #[test]
    fn test_union_appends_all_cell_types() {
        let mut a = PolyData::from_vertices(vec![[0.0, 0.0, 0.0]]);
        a.lines.push_cell(&[0]);
        let b = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_union_append(&a, &b);
        assert_eq!(r.points.len(), 4);
        assert_eq!(r.verts.num_cells(), 1);
        assert_eq!(r.lines.num_cells(), 1);
        assert_eq!(r.polys.num_cells(), 1);
        assert_eq!(r.polys.cell(0), &[1, 2, 3]);
    }
    #[test]
    fn test_xor() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.5, 0.0, 0.0], [1.5, 0.0, 0.0], [1.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boolean_xor_simple(&a, &b);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
