use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Catmull-Clark subdivision for quad-dominant meshes.
///
/// Each face is split by inserting a face point (centroid), edge midpoints,
/// and updating original vertices. Triangles are first treated as degenerate
/// quads. Produces an all-quad output after one iteration.
pub fn catmull_clark(input: &PolyData) -> PolyData {
    let n_pts = input.points.len();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_faces = cells.len();

    // 1. Compute face points (centroids)
    let mut face_points: Vec<[f64; 3]> = Vec::with_capacity(n_faces);
    for cell in &cells {
        let mut c = [0.0; 3];
        for &id in cell {
            let p = input.points.get(id as usize);
            c[0] += p[0];
            c[1] += p[1];
            c[2] += p[2];
        }
        let n = cell.len() as f64;
        face_points.push([c[0] / n, c[1] / n, c[2] / n]);
    }

    // 2. Build edge-face adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // 3. Compute edge points: avg of edge endpoints + adjacent face points
    let mut edge_point_map: HashMap<(i64, i64), usize> = HashMap::new();
    let _new_points: Vec<[f64; 3]> = Vec::new();

    // Reserve slots for: original points (updated), face points, edge points
    // Start: copy original points (will update later)
    let mut out_pts: Vec<[f64; 3]> = (0..n_pts).map(|i| input.points.get(i)).collect();

    // Add face points
    let face_pt_start = out_pts.len();
    out_pts.extend_from_slice(&face_points);

    // Add edge points
    let _edge_pt_start = out_pts.len();
    for (&(a, b), faces) in &edge_faces {
        let pa = input.points.get(a as usize);
        let pb = input.points.get(b as usize);
        let ep = if faces.len() == 2 {
            let fp0 = face_points[faces[0]];
            let fp1 = face_points[faces[1]];
            [
                (pa[0] + pb[0] + fp0[0] + fp1[0]) / 4.0,
                (pa[1] + pb[1] + fp0[1] + fp1[1]) / 4.0,
                (pa[2] + pb[2] + fp0[2] + fp1[2]) / 4.0,
            ]
        } else {
            [
                (pa[0] + pb[0]) / 2.0,
                (pa[1] + pb[1]) / 2.0,
                (pa[2] + pb[2]) / 2.0,
            ]
        };
        let idx = out_pts.len();
        out_pts.push(ep);
        edge_point_map.insert((a, b), idx);
    }

    // 4. Update original vertex positions
    // For each original vertex: new_pos = (F + 2R + (n-3)P) / n
    // F = avg of adjacent face points, R = avg of adjacent edge midpoints, P = original, n = valence
    let mut pt_faces: Vec<Vec<usize>> = vec![Vec::new(); n_pts];
    let mut pt_edges: Vec<Vec<(i64, i64)>> = vec![Vec::new(); n_pts];
    for (fi, cell) in cells.iter().enumerate() {
        for &id in cell {
            pt_faces[id as usize].push(fi);
        }
    }
    for &(a, b) in edge_faces.keys() {
        pt_edges[a as usize].push((a, b));
        pt_edges[b as usize].push((a, b));
    }

    for i in 0..n_pts {
        let valence = pt_faces[i].len() as f64;
        if valence < 1.0 {
            continue;
        }

        let mut f_avg = [0.0; 3];
        for &fi in &pt_faces[i] {
            f_avg[0] += face_points[fi][0];
            f_avg[1] += face_points[fi][1];
            f_avg[2] += face_points[fi][2];
        }
        f_avg[0] /= valence;
        f_avg[1] /= valence;
        f_avg[2] /= valence;

        let mut r_avg = [0.0; 3];
        let mut r_count = 0.0;
        for &(a, b) in &pt_edges[i] {
            let pa = input.points.get(a as usize);
            let pb = input.points.get(b as usize);
            r_avg[0] += (pa[0] + pb[0]) / 2.0;
            r_avg[1] += (pa[1] + pb[1]) / 2.0;
            r_avg[2] += (pa[2] + pb[2]) / 2.0;
            r_count += 1.0;
        }
        if r_count > 0.0 {
            r_avg[0] /= r_count;
            r_avg[1] /= r_count;
            r_avg[2] /= r_count;
        }

        let p = input.points.get(i);
        let n = valence;
        out_pts[i] = [
            (f_avg[0] + 2.0 * r_avg[0] + (n - 3.0) * p[0]) / n,
            (f_avg[1] + 2.0 * r_avg[1] + (n - 3.0) * p[1]) / n,
            (f_avg[2] + 2.0 * r_avg[2] + (n - 3.0) * p[2]) / n,
        ];
    }

    // 5. Create new quad faces
    let mut out_polys = CellArray::new();
    let get_edge_pt = |a: i64, b: i64| -> i64 {
        let key = if a < b { (a, b) } else { (b, a) };
        edge_point_map[&key] as i64
    };

    for (fi, cell) in cells.iter().enumerate() {
        let fp = (face_pt_start + fi) as i64;
        let n = cell.len();
        for i in 0..n {
            let v = cell[i];
            let ep_prev = get_edge_pt(cell[(i + n - 1) % n], v);
            let ep_next = get_edge_pt(v, cell[(i + 1) % n]);
            out_polys.push_cell(&[v, ep_next, fp, ep_prev]);
        }
    }

    let mut points = Points::<f64>::new();
    for p in &out_pts {
        points.push(*p);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivide_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = catmull_clark(&pd);
        assert_eq!(result.polys.num_cells(), 4); // 1 quad -> 4 quads
    }

    #[test]
    fn subdivide_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = catmull_clark(&pd);
        assert_eq!(result.polys.num_cells(), 3); // 1 tri -> 3 quads
    }

    #[test]
    fn two_quads() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 4, 3]);
        pd.polys.push_cell(&[1, 2, 5, 4]);

        let result = catmull_clark(&pd);
        assert_eq!(result.polys.num_cells(), 8); // 2 -> 8
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = catmull_clark(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
