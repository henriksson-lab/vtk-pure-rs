//! Compute minimum distance from each vertex to any non-adjacent edge.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn vertex_edge_distance(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Collect all edges
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut vert_adj: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                let e = if a < b { (a, b) } else { (b, a) };
                vert_adj[a].insert(b);
                vert_adj[b].insert(a);
                edges.push(e);
            }
        }
    }
    edges.sort();
    edges.dedup();
    let mut min_dist = vec![f64::INFINITY; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        for &(a, b) in &edges {
            if a == i || b == i {
                continue;
            } // skip adjacent
            if vert_adj[i].contains(&a) && vert_adj[i].contains(&b) {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let ap = [p[0] - pa[0], p[1] - pa[1], p[2] - pa[2]];
            let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            if ab2 < 1e-15 {
                continue;
            }
            let t = (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2;
            let t = t.clamp(0.0, 1.0);
            let closest = [pa[0] + t * ab[0], pa[1] + t * ab[1], pa[2] + t * ab[2]];
            let d = ((p[0] - closest[0]).powi(2)
                + (p[1] - closest[1]).powi(2)
                + (p[2] - closest[2]).powi(2))
            .sqrt();
            if d < min_dist[i] {
                min_dist[i] = d;
            }
        }
        if min_dist[i] == f64::INFINITY {
            min_dist[i] = 0.0;
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "EdgeDistance",
            min_dist,
            1,
        )));
    result.point_data_mut().set_active_scalars("EdgeDistance");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_edge_dist() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [3.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = vertex_edge_distance(&mesh);
        assert!(r.point_data().get_array("EdgeDistance").is_some());
    }
}
