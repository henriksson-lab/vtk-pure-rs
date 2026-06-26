use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Build the dual graph of a triangle mesh.
///
/// Each face becomes a node, each shared edge becomes a graph edge.
/// Returns a PolyData where points are face centroids and lines are
/// connections between adjacent faces. Adds "FaceIndex" point data.
pub fn dual_graph(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = cells.len();

    // Face centroids
    let mut out_points = Points::<f64>::new();
    let mut face_ids = Vec::with_capacity(n_cells);
    for (fi, cell) in cells.iter().enumerate() {
        if cell.is_empty() {
            out_points.push([0.0; 3]);
            face_ids.push(fi as f64);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let mut n = 0.0;
        for &id in cell {
            let Ok(id) = usize::try_from(id) else {
                continue;
            };
            if id >= input.points.len() {
                continue;
            }
            let p = input.points.get(id);
            cx += p[0];
            cy += p[1];
            cz += p[2];
            n += 1.0;
        }
        if n == 0.0 {
            out_points.push([0.0; 3]);
        } else {
            out_points.push([cx / n, cy / n, cz / n]);
        }
        face_ids.push(fi as f64);
    }

    // Edge adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            if a < 0
                || b < 0
                || a as usize >= input.points.len()
                || b as usize >= input.points.len()
            {
                continue;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut out_lines = CellArray::new();
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            out_lines.push_cell(&[faces[0] as i64, faces[1] as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceIndex",
            face_ids,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_of_two_tris() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = dual_graph(&pd);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.lines.num_cells(), 1); // one shared edge
    }

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = dual_graph(&pd);
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.lines.num_cells(), 0); // no adjacency
    }

    #[test]
    fn has_face_index() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = dual_graph(&pd);
        assert!(result.point_data().get_array("FaceIndex").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = dual_graph(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
