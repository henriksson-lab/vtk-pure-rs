//! Count boundary edges incident to each vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn boundary_edge_count(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    let mut bcount = vec![0.0f64; n];
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            bcount[a] += 1.0;
            bcount[b] += 1.0;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BoundaryEdgeCount",
            bcount,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("BoundaryEdgeCount");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_boundary_count() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = boundary_edge_count(&mesh);
        let arr = r.point_data().get_array("BoundaryEdgeCount").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 2.0); // vertex 0 has 2 boundary edges
    }
}
