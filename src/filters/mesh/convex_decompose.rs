use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Approximate convex decomposition by grouping faces.
///
/// Grows convex patches from seeds: a patch is convex if all edges
/// between faces in the patch are convex (dihedral angle < 180°).
/// Adds "ConvexPatch" cell data.
pub fn convex_decompose(input: &PolyData) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return input.clone();
    }

    let center = mesh_centroid(input);
    let normals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            let Some(origin) = first_valid_point(input, c) else {
                return [0.0; 3];
            };
            face_normal(input, c)
                .map(|normal| orient_normal(origin, normal, center))
                .unwrap_or([0.0; 3])
        })
        .collect();

    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    // Check if edge between two faces is convex
    let is_convex_edge = |fi: usize, fj: usize, edge: &(i64, i64)| -> bool {
        let na = normals[fi];
        // Find opposite vertex in fj
        let opp = cells[fj].iter().find(|&&v| v != edge.0 && v != edge.1);
        if let Some(&ov) = opp {
            let Some(ov) = valid_point_id(input, ov) else {
                return true;
            };
            let Some(edge0) = valid_point_id(input, edge.0) else {
                return true;
            };
            let p = input.points.get(ov);
            let v0 = input.points.get(edge0);
            let d = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];
            d[0] * na[0] + d[1] * na[1] + d[2] * na[2] <= 0.01 // convex if opp vertex is below plane
        } else {
            true
        }
    };

    let mut adj: Vec<Vec<(usize, (i64, i64))>> = vec![Vec::new(); nc];
    for (&edge, faces) in &edge_faces {
        if faces.len() == 2 {
            adj[faces[0]].push((faces[1], edge));
            adj[faces[1]].push((faces[0], edge));
        }
    }

    let mut labels = vec![0usize; nc];
    let mut current = 0;

    for seed in 0..nc {
        if labels[seed] != 0 {
            continue;
        }
        current += 1;
        labels[seed] = current;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(seed);

        while let Some(fi) = queue.pop_front() {
            for &(ni, edge) in &adj[fi] {
                if labels[ni] != 0 {
                    continue;
                }
                if is_convex_edge(fi, ni, &edge) {
                    labels[ni] = current;
                    queue.push_back(ni);
                }
            }
        }
    }

    let labels_f: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConvexPatch",
            labels_f,
            1,
        )));
    pd
}

fn face_normal(input: &PolyData, cell: &[i64]) -> Option<[f64; 3]> {
    let origin = first_valid_point(input, cell)?;
    for i in 1..cell.len().saturating_sub(1) {
        let v1 = input.points.get(valid_point_id(input, cell[i])?);
        let v2 = input.points.get(valid_point_id(input, cell[i + 1])?);
        let e1 = [v1[0] - origin[0], v1[1] - origin[1], v1[2] - origin[2]];
        let e2 = [v2[0] - origin[0], v2[1] - origin[1], v2[2] - origin[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 1e-15 {
            return Some([n[0] / l, n[1] / l, n[2] / l]);
        }
    }
    None
}

fn first_valid_point(input: &PolyData, cell: &[i64]) -> Option<[f64; 3]> {
    let point_id = *cell.first()?;
    valid_point_id(input, point_id).map(|point_id| input.points.get(point_id))
}

fn valid_point_id(input: &PolyData, point_id: i64) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < input.points.len())
}

fn mesh_centroid(input: &PolyData) -> [f64; 3] {
    if input.points.len() == 0 {
        return [0.0; 3];
    }

    let mut center = [0.0; 3];
    for point_id in 0..input.points.len() {
        let p = input.points.get(point_id);
        center[0] += p[0];
        center[1] += p[1];
        center[2] += p[2];
    }
    let inv_n = 1.0 / input.points.len() as f64;
    [center[0] * inv_n, center[1] * inv_n, center[2] * inv_n]
}

fn orient_normal(origin: [f64; 3], normal: [f64; 3], center: [f64; 3]) -> [f64; 3] {
    let d = [
        center[0] - origin[0],
        center[1] - origin[1],
        center[2] - origin[2],
    ];
    if d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2] > 0.0 {
        [-normal[0], -normal[1], -normal[2]]
    } else {
        normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_one_patch() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = convex_decompose(&pd);
        let arr = result.cell_data().get_array("ConvexPatch").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let p0 = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(p0, buf[0]); // same patch (flat = convex)
    }

    #[test]
    fn has_patches() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.5]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = convex_decompose(&pd);
        assert!(result.cell_data().get_array("ConvexPatch").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = convex_decompose(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn convex_tetrahedron_stays_one_patch_with_mixed_winding() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = convex_decompose(&pd);
        let arr = result.cell_data().get_array("ConvexPatch").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let patch = buf[0];
        for cell_id in 1..arr.num_tuples() {
            arr.tuple_as_f64(cell_id, &mut buf);
            assert_eq!(buf[0], patch);
        }
    }
}
