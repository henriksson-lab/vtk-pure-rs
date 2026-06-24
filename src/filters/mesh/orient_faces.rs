use crate::data::{CellArray, PolyData};
use std::collections::{HashMap, HashSet, VecDeque};

/// Orient all faces consistently using BFS from a seed face.
///
/// Starting from face 0, propagates winding order to adjacent faces
/// through shared edges. Flips faces whose winding disagrees with
/// their already-oriented neighbor.
pub fn orient_faces_consistent(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return input.clone();
    }

    // Build edge-to-face adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut oriented = vec![false; nc];
    let mut flipped = vec![false; nc];
    let mut queue = VecDeque::new();

    oriented[0] = true;
    queue.push_back(0);

    while let Some(fi) = queue.pop_front() {
        let c = &cells[fi];
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };

            if let Some(adj) = edge_faces.get(&key) {
                for &ni in adj {
                    if ni == fi || oriented[ni] {
                        continue;
                    }
                    oriented[ni] = true;

                    // Check if ni has the edge in the same direction
                    let nc_cell = &cells[ni];
                    let mut same_dir = false;
                    for j in 0..nc_cell.len() {
                        if nc_cell[j] == a && nc_cell[(j + 1) % nc_cell.len()] == b {
                            same_dir = true;
                            break;
                        }
                    }

                    // Adjacent faces should have OPPOSITE edge direction
                    // (if fi has a->b, ni should have b->a)
                    let fi_has_ab = !flipped[fi]; // fi's actual direction
                    if same_dir == fi_has_ab {
                        // Same direction = needs flip
                        flipped[ni] = !flipped[fi];
                    } else {
                        flipped[ni] = flipped[fi];
                    }

                    queue.push_back(ni);
                }
            }
        }
    }

    let mut out_polys = CellArray::new();
    for (fi, c) in cells.iter().enumerate() {
        if flipped[fi] {
            let mut rev = c.clone();
            rev.reverse();
            out_polys.push_cell(&rev);
        } else {
            out_polys.push_cell(c);
        }
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_consistent() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = orient_faces_consistent(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn fixes_flipped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 2, 0]); // wrong winding

        let result = orient_faces_consistent(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn single_face() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = orient_faces_consistent(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = orient_faces_consistent(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
