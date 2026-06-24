use crate::data::{CellArray, Points, PolyData};

/// Dual contouring on a mesh: create a new mesh where each face becomes a vertex
/// and each vertex becomes a face connecting adjacent face-vertices.
///
/// This is the topological dual, different from `dual_mesh` which only
/// creates the dual graph. Here, actual polygonal cells are produced.
pub fn dual_contour_mesh(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return PolyData::new();
    }

    // Face centroids become vertices
    let mut out_pts = Points::<f64>::new();
    for c in &cells {
        if c.is_empty() {
            out_pts.push([0.0; 3]);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in c {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n_v = c.len() as f64;
        out_pts.push([cx / n_v, cy / n_v, cz / n_v]);
    }

    // For each original vertex with 3+ adjacent faces, create a dual polygon
    let mut pt_faces: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (fi, c) in cells.iter().enumerate() {
        for &v in c {
            pt_faces[v as usize].push(fi);
        }
    }

    let mut out_polys = CellArray::new();
    for i in 0..n {
        let adj = &pt_faces[i];
        if adj.len() < 3 {
            continue;
        }

        // Order faces around vertex by angle
        let p = input.points.get(i);
        let mut angles: Vec<(usize, f64)> = adj
            .iter()
            .map(|&fi| {
                let c = out_pts.get(fi);
                let dx = c[0] - p[0];
                let dy = c[1] - p[1];
                (fi, dy.atan2(dx))
            })
            .collect();
        angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let face_ids: Vec<i64> = angles.iter().map(|&(fi, _)| fi as i64).collect();
        out_polys.push_cell(&face_ids);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_of_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = dual_contour_mesh(&pd);
        assert_eq!(result.points.len(), 2); // 2 face centroids
    }

    #[test]
    fn dual_of_fan() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..5 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 5.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..5 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 5 + 1) as i64]);
        }

        let result = dual_contour_mesh(&pd);
        assert_eq!(result.points.len(), 5); // 5 face centroids
        assert!(result.polys.num_cells() >= 1); // center vertex -> 5-gon
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(dual_contour_mesh(&pd).polys.num_cells(), 0);
    }
}
