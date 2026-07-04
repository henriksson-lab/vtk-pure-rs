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
        add_polygon_edges(cell, n, &mut edge_count);
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(strip, n, &mut edge_count);
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

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

fn add_polygon_edges(
    cell: &[i64],
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    let nc = cell.len();
    if nc < 2 || !valid_cell(cell, npoints) {
        return;
    }
    for i in 0..nc {
        add_edge(cell[i] as usize, cell[(i + 1) % nc] as usize, edge_count);
    }
}

fn add_triangle_strip_edges(
    strip: &[i64],
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), u32>,
) {
    if strip.len() < 3 || !valid_cell(strip, npoints) {
        return;
    }
    for i in 0..strip.len() - 2 {
        let tri = if i % 2 == 0 {
            [strip[i], strip[i + 1], strip[i + 2]]
        } else {
            [strip[i + 1], strip[i], strip[i + 2]]
        };
        add_edge(tri[0] as usize, tri[1] as usize, edge_count);
        add_edge(tri[1] as usize, tri[2] as usize, edge_count);
        add_edge(tri[2] as usize, tri[0] as usize, edge_count);
    }
}

fn add_edge(a: usize, b: usize, edge_count: &mut std::collections::HashMap<(usize, usize), u32>) {
    if a == b {
        return;
    }
    let e = if a < b { (a, b) } else { (b, a) };
    *edge_count.entry(e).or_insert(0) += 1;
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

    #[test]
    fn triangle_strips_are_decomposed() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let r = boundary_edge_count(&mesh);
        let arr = r.point_data().get_array("BoundaryEdgeCount").unwrap();

        assert_eq!(arr.to_f64_vec(), vec![2.0, 2.0, 2.0, 2.0]);
    }
}
