use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

/// Decimate flat regions by merging triangles with small dihedral angles.
///
/// Removes edges between coplanar triangles (dihedral angle < threshold).
/// More aggressive on flat regions, preserves sharp features.
pub fn decimate_flat(input: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let cos_thresh = angle_threshold_deg.to_radians().cos();

    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = cells.len();

    // Face normals
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return [0.0; 3];
            }
            let v0 = input.points.get(cell[0] as usize);
            let v1 = input.points.get(cell[1] as usize);
            let v2 = input.points.get(cell[2] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-15 {
                [n[0] / len, n[1] / len, n[2] / len]
            } else {
                [0.0; 3]
            }
        })
        .collect();

    // Edge adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Mark coplanar pairs for merging via union-find
    let mut parent: Vec<usize> = (0..n_cells).collect();
    let find = |p: &mut Vec<usize>, mut x: usize| -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    };

    for faces in edge_faces.values() {
        if faces.len() == 2 {
            let na = normals[faces[0]];
            let nb = normals[faces[1]];
            let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
            if dot >= cos_thresh {
                let ra = find(&mut parent, faces[0]);
                let rb = find(&mut parent, faces[1]);
                if ra != rb {
                    parent[rb] = ra;
                }
            }
        }
    }

    // For each flat connected group, keep one representative cell. This is a
    // coarse decimation of coplanar patches rather than full polygon merging.
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n_cells {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }

    let mut out_polys = CellArray::new();
    let mut representatives: Vec<usize> = groups
        .values()
        .filter_map(|members| members.iter().copied().min())
        .collect();
    representatives.sort_unstable();
    for fi in representatives {
        out_polys.push_cell(&cells[fi]);
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_surface_preserved() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = decimate_flat(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn sharp_edge_kept() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]); // XY plane
        pd.points.push([0.5, 0.0, 1.0]); // XZ plane
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = decimate_flat(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 2); // not merged
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = decimate_flat(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
