//! Compute Voronoi cell area per vertex (dual area).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the Voronoi area (dual area) for each vertex.
///
/// Each vertex's Voronoi area is the sum of 1/3 of each adjacent triangle's area.
pub fn voronoi_vertex_area(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut areas = vec![0.0f64; n];

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let tri_area = 0.5
            * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
        let share = tri_area / cell.len() as f64;
        for &pid in cell {
            areas[pid as usize] += share;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiArea",
            areas,
            1,
        )));
    result
}

/// Compute area-weighted centroid per vertex (Voronoi centroid).
pub fn voronoi_centroids(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let adj = build_adj(mesh, n);
    let mut centroids = Vec::with_capacity(n * 3);

    for i in 0..n {
        if adj[i].is_empty() {
            let p = mesh.points.get(i);
            centroids.extend_from_slice(&p);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &j in &adj[i] {
            let p = mesh.points.get(j);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let k = adj[i].len() as f64;
        centroids.push(cx / k);
        centroids.push(cy / k);
        centroids.push(cz / k);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiCentroid",
            centroids,
            3,
        )));
    result
}

fn build_adj(m: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for c in m.polys.iter() {
        let nc = c.len();
        for i in 0..nc {
            let x = c[i] as usize;
            let y = c[(i + 1) % nc] as usize;
            if x < n && y < n {
                a[x].insert(y);
                a[y].insert(x);
            }
        }
    }
    a.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = voronoi_vertex_area(&mesh);
        let arr = result.point_data().get_array("VoronoiArea").unwrap();
        let mut buf = [0.0f64];
        let mut total = 0.0;
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            total += buf[0];
        }
        assert!((total - 0.5).abs() < 0.01); // total should equal triangle area
    }
    #[test]
    fn centroids() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = voronoi_centroids(&mesh);
        assert!(result.point_data().get_array("VoronoiCentroid").is_some());
    }
}
