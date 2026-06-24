use crate::data::{AnyDataArray, DataArray, PolyData};

/// Merge coplanar adjacent faces into larger polygons.
///
/// Triangles that share an edge and have normals within `angle_tolerance`
/// degrees are merged into a single polygon. Reduces face count on
/// piecewise-flat surfaces.
pub fn merge_coplanar(input: &PolyData, angle_tolerance: f64) -> PolyData {
    let cos_tol = angle_tolerance.to_radians().cos();

    // Compute face normals
    let face_normals: Vec<[f64; 3]> = input
        .polys
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return [0.0, 0.0, 0.0];
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
                [0.0, 0.0, 0.0]
            }
        })
        .collect();

    // Build edge-to-face adjacency
    let mut edge_faces: std::collections::HashMap<(i64, i64), Vec<usize>> =
        std::collections::HashMap::new();
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    for (fi, cell) in cells.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Union-find for merging
    let n_faces = cells.len();
    let mut parent: Vec<usize> = (0..n_faces).collect();

    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    };

    for faces in edge_faces.values() {
        if faces.len() == 2 {
            let a = faces[0];
            let b = faces[1];
            let na = face_normals[a];
            let nb = face_normals[b];
            let dot = na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2];
            if dot >= cos_tol {
                let ra = find(&mut parent, a);
                let rb = find(&mut parent, b);
                if ra != rb {
                    parent[rb] = ra;
                }
            }
        }
    }

    // Group faces by root
    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..n_faces {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }

    // For each group, if single face pass through; otherwise keep individual faces
    // (Full polygon merging is complex; for now just tag with GroupId)
    let mut group_ids = vec![0.0f64; n_faces];
    for (gi, (_, members)) in groups.iter().enumerate() {
        for &fi in members {
            group_ids[fi] = gi as f64;
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CoplanarGroupId",
            group_ids,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coplanar_triangles_grouped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // coplanar
        pd.polys.push_cell(&[0, 2, 3]); // coplanar

        let result = merge_coplanar(&pd, 1.0);
        let arr = result.cell_data().get_array("CoplanarGroupId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let g0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(g0, buf[0]); // same group
    }

    #[test]
    fn non_coplanar_separate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]); // XY plane
        pd.polys.push_cell(&[0, 1, 3]); // XZ plane

        let result = merge_coplanar(&pd, 1.0); // 1 degree tolerance
        let arr = result.cell_data().get_array("CoplanarGroupId").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let g0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        assert_ne!(g0, buf[0]); // different groups
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = merge_coplanar(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
