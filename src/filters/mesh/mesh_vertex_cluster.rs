//! Vertex clustering decimation using a uniform grid.
use crate::data::{CellArray, Points, PolyData};

pub fn vertex_cluster(mesh: &PolyData, grid_size: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || grid_size <= 0.0 {
        return mesh.clone();
    }
    let inv = 1.0 / grid_size;
    // Assign each vertex to a grid cell
    let mut cell_map: std::collections::HashMap<(i64, i64, i64), (usize, [f64; 3], usize)> =
        std::collections::HashMap::new();
    let mut vertex_to_cluster = vec![0usize; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        let key = (
            (p[0] * inv).floor() as i64,
            (p[1] * inv).floor() as i64,
            (p[2] * inv).floor() as i64,
        );
        let next_id = cell_map.len();
        let entry = cell_map.entry(key).or_insert((next_id, [0.0, 0.0, 0.0], 0));
        entry.1[0] += p[0];
        entry.1[1] += p[1];
        entry.1[2] += p[2];
        entry.2 += 1;
        vertex_to_cluster[i] = entry.0;
    }
    let mut pts = Points::<f64>::new();
    let mut cluster_to_pt = std::collections::HashMap::new();
    for (_, &(id, sum, count)) in &cell_map {
        let c = count as f64;
        cluster_to_pt.insert(id, pts.len());
        pts.push([sum[0] / c, sum[1] / c, sum[2] / c]);
    }
    let mut polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let mapped: Vec<i64> = cell
            .iter()
            .map(|&v| {
                let cluster = vertex_to_cluster[v as usize];
                *cluster_to_pt.get(&cluster).unwrap() as i64
            })
            .collect();
        // Skip degenerate cells
        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= 3 {
            polys.push_cell(&mapped);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cluster() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.05, 0.1, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.1, 0.0],
                [0.95, 0.05, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = vertex_cluster(&mesh, 0.5);
        assert!(r.points.len() <= mesh.points.len());
    }
}
