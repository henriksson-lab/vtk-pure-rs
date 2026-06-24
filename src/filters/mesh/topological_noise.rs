use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Remove topological noise: small connected components by face count.
///
/// Removes connected components with fewer than `min_faces` triangles.
pub fn remove_small_components(input: &PolyData, min_faces: usize) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return input.clone();
    }

    // Union-find by shared vertices
    let mut parent: Vec<usize> = (0..nc).collect();
    let find = |p: &mut Vec<usize>, mut x: usize| -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    };

    let mut vert_cell: HashMap<i64, usize> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for &v in c {
            if let Some(&prev_fi) = vert_cell.get(&v) {
                let ra = find(&mut parent, fi);
                let rb = find(&mut parent, prev_fi);
                if ra != rb {
                    parent[rb] = ra;
                }
            }
            vert_cell.insert(v, fi);
        }
    }

    // Count faces per component
    let mut comp_size: HashMap<usize, usize> = HashMap::new();
    for i in 0..nc {
        *comp_size.entry(find(&mut parent, i)).or_insert(0) += 1;
    }

    // Keep only large components
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for (fi, c) in cells.iter().enumerate() {
        let root = find(&mut parent, fi);
        if comp_size[&root] >= min_faces {
            let mapped: Vec<i64> = c
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
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
    fn remove_small() {
        let mut pd = PolyData::new();
        // Large component: 2 triangles
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        // Small component: 1 triangle
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.polys.push_cell(&[4, 5, 6]);

        let result = remove_small_components(&pd, 2);
        assert_eq!(result.polys.num_cells(), 2); // small removed
    }

    #[test]
    fn keep_all_large() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remove_small_components(&pd, 1);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = remove_small_components(&pd, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
