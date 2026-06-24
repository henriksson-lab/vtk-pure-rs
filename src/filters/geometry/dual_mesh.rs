use crate::data::{CellArray, Points, PolyData};

/// Compute the dual mesh of a triangle mesh.
///
/// Each triangle becomes a vertex (at its centroid), and each original
/// vertex becomes a polygon connecting the centroids of its adjacent triangles.
/// The dual of a triangle mesh is a polygon mesh (mostly hexagons for regular meshes).
pub fn dual_mesh(input: &PolyData) -> PolyData {
    let n_pts = input.points.len();

    // Compute triangle centroids
    let mut centroids: Vec<[f64; 3]> = Vec::new();
    let mut tri_indices: Vec<[i64; 3]> = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        centroids.push([
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ]);
        tri_indices.push([cell[0], cell[1], cell[2]]);
    }

    // For each original vertex, find which triangles contain it
    let mut pt_tris: Vec<Vec<usize>> = vec![Vec::new(); n_pts];
    for (ti, tri) in tri_indices.iter().enumerate() {
        for &v in tri {
            pt_tris[v as usize].push(ti);
        }
    }

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Add all centroids as points
    for c in &centroids {
        out_points.push(*c);
    }

    // For each vertex with ≥3 adjacent triangles, create a dual polygon
    for pi in 0..n_pts {
        let adj = &pt_tris[pi];
        if adj.len() < 3 {
            continue;
        }

        // Order by angle around the vertex
        let p = input.points.get(pi);
        let mut angles: Vec<(usize, f64)> = adj
            .iter()
            .map(|&ti| {
                let c = centroids[ti];
                let dx = c[0] - p[0];
                let dy = c[1] - p[1];
                (ti, dy.atan2(dx))
            })
            .collect();
        angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let cell: Vec<i64> = angles.iter().map(|&(ti, _)| ti as i64).collect();
        out_polys.push_cell(&cell);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_of_two_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = dual_mesh(&pd);
        assert_eq!(result.points.len(), 2); // 2 triangle centroids
    }

    #[test]
    fn dual_of_fan() {
        // Central vertex surrounded by 5 triangles
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..5 {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / 5.0;
            pd.points.push([angle.cos(), angle.sin(), 0.0]);
        }
        for i in 0..5 {
            let j = (i + 1) % 5;
            pd.polys.push_cell(&[0, (i + 1) as i64, (j + 1) as i64]);
        }

        let result = dual_mesh(&pd);
        assert_eq!(result.points.len(), 5); // 5 centroids
        assert!(result.polys.num_cells() >= 1); // at least the center vertex polygon
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = dual_mesh(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
