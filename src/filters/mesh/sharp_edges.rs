use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Extract edges where the dihedral angle exceeds a threshold (sharp/feature edges).
///
/// Returns a PolyData with line cells for sharp edges only.
///
/// This mirrors `vtkFeatureEdges` configured with `FeatureEdgesOn` and
/// `BoundaryEdgesOff`/`NonManifoldEdgesOff`/`ManifoldEdgesOff`: only edges shared
/// by exactly two polygons are considered, and an edge is emitted when
/// `dot(n0, n1) <= cos(feature_angle)` (VTK's `vtkFeatureEdges.cxx:438`).
/// The full multi-category filter lives in [`crate::filters::geometry::feature_edges`].
pub fn extract_sharp_edges(input: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let cos_thresh = angle_threshold_deg.to_radians().cos();

    let cells: Vec<Vec<i64>> = input
        .polys
        .iter()
        .filter(|c| valid_cell(input, c))
        .map(|c| c.to_vec())
        .collect();
    let normals: Vec<[f64; 3]> = cells.iter().map(|c| face_normal(input, c)).collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    // Emit in a deterministic edge order (hash iteration order is not stable).
    let mut edges: Vec<((i64, i64), Vec<usize>)> = edge_faces.into_iter().collect();
    edges.sort_by_key(|&((a, b), _)| (a, b));

    for ((a, b), faces) in edges {
        if faces.len() != 2 {
            continue;
        }
        let na = normals[faces[0]];
        let nb = normals[faces[1]];
        let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
        if dot <= cos_thresh {
            let ma = *pt_map.entry(a).or_insert_with(|| {
                let i = out_pts.len() as i64;
                out_pts.push(input.points.get(a as usize));
                i
            });
            let mb = *pt_map.entry(b).or_insert_with(|| {
                let i = out_pts.len() as i64;
                out_pts.push(input.points.get(b as usize));
                i
            });
            out_lines.push_cell(&[ma, mb]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.lines = out_lines;
    pd
}

/// Mark vertices that lie on sharp edges. Adds "SharpVertex" scalar (0 or 1).
pub fn mark_sharp_vertices(input: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let n = input.points.len();
    let mut is_sharp = vec![0.0f64; n];

    // Map sharp edge points back to original indices
    let cos_thresh = angle_threshold_deg.to_radians().cos();
    let cells: Vec<Vec<i64>> = input
        .polys
        .iter()
        .filter(|c| valid_cell(input, c))
        .map(|c| c.to_vec())
        .collect();
    let normals: Vec<[f64; 3]> = cells.iter().map(|c| face_normal(input, c)).collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    for (&(a, b), faces) in &edge_faces {
        if faces.len() != 2 {
            continue;
        }
        let na = normals[faces[0]];
        let nb = normals[faces[1]];
        let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
        if dot <= cos_thresh {
            is_sharp[a as usize] = 1.0;
            is_sharp[b as usize] = 1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SharpVertex",
            is_sharp,
            1,
        )));
    pd
}

fn valid_cell(input: &PolyData, cell: &[i64]) -> bool {
    cell.len() >= 3
        && cell
            .iter()
            .all(|&point_id| point_id >= 0 && (point_id as usize) < input.points.len())
}

fn face_normal(input: &PolyData, c: &[i64]) -> [f64; 3] {
    let mut n = [0.0, 0.0, 0.0];
    for i in 0..c.len() {
        let current = input.points.get(c[i] as usize);
        let next = input.points.get(c[(i + 1) % c.len()] as usize);
        n[0] += (current[1] - next[1]) * (current[2] + next[2]);
        n[1] += (current[2] - next[2]) * (current[0] + next[0]);
        n[2] += (current[0] - next[0]) * (current[1] + next[1]);
    }
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0; 3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sharp_90_degree_edge() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = extract_sharp_edges(&pd, 45.0);
        assert!(result.lines.num_cells() >= 1);
    }

    #[test]
    fn flat_no_sharp() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = extract_sharp_edges(&pd, 10.0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn mark_sharp() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = mark_sharp_vertices(&pd, 45.0);
        assert!(result.point_data().get_array("SharpVertex").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(extract_sharp_edges(&pd, 30.0).lines.num_cells(), 0);
    }

    #[test]
    fn handles_polygon_with_collinear_first_three_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2, 3, 4]);
        pd.polys.push_cell(&[1, 5, 6, 2]);

        let result = extract_sharp_edges(&pd, 45.0);

        assert_eq!(result.lines.num_cells(), 1);
    }
}
