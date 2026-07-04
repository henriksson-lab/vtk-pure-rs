//! Compute minimum distance from each vertex to any non-adjacent edge.
use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

pub fn vertex_edge_distance(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut vert_adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];

    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            insert_edge(&mut edges, &mut vert_adj, n, edge[0], edge[1]);
        }
    }

    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            insert_edge(
                &mut edges,
                &mut vert_adj,
                n,
                cell[i],
                cell[(i + 1) % cell.len()],
            );
        }
    }

    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            insert_edge(&mut edges, &mut vert_adj, n, tri[0], tri[1]);
            insert_edge(&mut edges, &mut vert_adj, n, tri[1], tri[2]);
            insert_edge(&mut edges, &mut vert_adj, n, tri[2], tri[0]);
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

fn insert_edge(
    edges: &mut Vec<(usize, usize)>,
    vert_adj: &mut [HashSet<usize>],
    n: usize,
    a: i64,
    b: i64,
) {
    let (Some(a), Some(b)) = (valid_point_index(a, n), valid_point_index(b, n)) else {
        return;
    };
    if a == b {
        return;
    }
    let e = if a < b { (a, b) } else { (b, a) };
    vert_adj[a].insert(b);
    vert_adj[b].insert(a);
    edges.push(e);
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
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
