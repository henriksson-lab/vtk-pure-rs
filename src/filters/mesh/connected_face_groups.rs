use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Label face-connected groups (components via shared edges).
///
/// Unlike vertex-based connectivity, this requires shared edges.
/// Adds "FaceGroup" cell data.
pub fn label_face_groups(input: &PolyData) -> (PolyData, usize) {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return (input.clone(), 0);
    }

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            adj[faces[0]].push(faces[1]);
            adj[faces[1]].push(faces[0]);
        }
    }

    let mut labels = vec![0usize; nc];
    let mut current = 0;
    let mut visited = vec![false; nc];

    for start in 0..nc {
        if visited[start] {
            continue;
        }
        current += 1;
        let mut stack = vec![start];
        while let Some(fi) = stack.pop() {
            if visited[fi] {
                continue;
            }
            visited[fi] = true;
            labels[fi] = current;
            for &ni in &adj[fi] {
                if !visited[ni] {
                    stack.push(ni);
                }
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FaceGroup",
            labels_f,
            1,
        )));
    (pd, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_groups() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([5.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let (result, count) = label_face_groups(&pd);
        assert_eq!(count, 2);
        assert!(result.cell_data().get_array("FaceGroup").is_some());
    }

    #[test]
    fn connected_pair() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let (_, count) = label_face_groups(&pd);
        assert_eq!(count, 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (_, count) = label_face_groups(&pd);
        assert_eq!(count, 0);
    }
}
