use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Graph-color faces so no two adjacent faces share the same color.
///
/// Uses a greedy coloring algorithm on the face adjacency graph.
/// Adds "FaceColor" cell data with integer color IDs (typically 4-6 colors).
pub fn face_coloring(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = cells.len();

    // Build face adjacency via shared edges
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_cells];
    for faces in edge_faces.values() {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                adj[faces[i]].push(faces[j]);
                adj[faces[j]].push(faces[i]);
            }
        }
    }

    // Greedy graph coloring
    let mut colors = vec![0usize; n_cells];
    for fi in 0..n_cells {
        let mut used = std::collections::HashSet::new();
        for &ni in &adj[fi] {
            if colors[ni] > 0 {
                used.insert(colors[ni]);
            }
        }
        let mut c = 1;
        while used.contains(&c) {
            c += 1;
        }
        colors[fi] = c;
    }

    let color_f: Vec<f64> = colors.iter().map(|&c| c as f64).collect();
    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceColor",
            color_f,
            1,
        )));
    pd
}

/// Count the number of colors used in the face coloring.
pub fn chromatic_number(input: &PolyData) -> usize {
    let result = face_coloring(input);
    let arr = match result.cell_data().get_array("FaceColor") {
        Some(a) => a,
        None => return 0,
    };
    let mut buf = [0.0f64];
    let mut max_c = 0usize;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        max_c = max_c.max(buf[0] as usize);
    }
    max_c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_two_adjacent() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = face_coloring(&pd);
        let arr = result.cell_data().get_array("FaceColor").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let c0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let c1 = buf[0];
        assert_ne!(c0, c1); // adjacent faces must differ
    }

    #[test]
    fn chromatic_small() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(chromatic_number(&pd), 2);
    }

    #[test]
    fn single_face() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(chromatic_number(&pd), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(chromatic_number(&pd), 0);
    }
}
