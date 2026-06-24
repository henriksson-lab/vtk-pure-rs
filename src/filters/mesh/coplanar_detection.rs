//! Detect and merge coplanar face groups.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Detect groups of coplanar adjacent faces.
///
/// Returns cell data "CoplanarGroupId" labeling each face.
pub fn detect_coplanar_groups(mesh: &PolyData, angle_tolerance_degrees: f64) -> PolyData {
    let cos_tol = (angle_tolerance_degrees * std::f64::consts::PI / 180.0).cos();
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = all_cells.len();
    let normals: Vec<[f64; 3]> = all_cells.iter().map(|c| face_normal(mesh, c)).collect();

    let mut edge_cells: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            edge_cells.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    let mut labels = vec![usize::MAX; n_cells];
    let mut next = 0;
    for seed in 0..n_cells {
        if labels[seed] != usize::MAX {
            continue;
        }
        let mut q = std::collections::VecDeque::new();
        q.push_back(seed);
        labels[seed] = next;
        while let Some(ci) = q.pop_front() {
            let cell = &all_cells[ci];
            let nc = cell.len();
            for i in 0..nc {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                if let Some(nbs) = edge_cells.get(&(a.min(b), a.max(b))) {
                    for &ni in nbs {
                        if labels[ni] != usize::MAX {
                            continue;
                        }
                        let dot = normals[ci][0] * normals[ni][0]
                            + normals[ci][1] * normals[ni][1]
                            + normals[ci][2] * normals[ni][2];
                        if dot >= cos_tol {
                            labels[ni] = next;
                            q.push_back(ni);
                        }
                    }
                }
            }
        }
        next += 1;
    }

    let data: Vec<f64> = labels.iter().map(|&l| l as f64).collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CoplanarGroupId",
            data,
            1,
        )));
    result
}

/// Count coplanar groups.
pub fn count_coplanar_groups(mesh: &PolyData) -> usize {
    match mesh.cell_data().get_array("CoplanarGroupId") {
        Some(arr) => {
            let mut max_id = -1i64;
            let mut buf = [0.0f64];
            for i in 0..arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                max_id = max_id.max(buf[0] as i64);
            }
            if max_id >= 0 {
                (max_id + 1) as usize
            } else {
                0
            }
        }
        None => 0,
    }
}

fn face_normal(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-15 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flat_plane_one_group() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..3 {
            for x in 0..3 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..2 {
            for x in 0..2 {
                let bl = y * 3 + x;
                tris.push([bl, bl + 1, bl + 4]);
                tris.push([bl, bl + 4, bl + 3]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = detect_coplanar_groups(&mesh, 5.0);
        assert_eq!(count_coplanar_groups(&result), 1);
    }
    #[test]
    fn cube_six_groups() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        );
        let result = detect_coplanar_groups(&mesh, 5.0);
        assert_eq!(count_coplanar_groups(&result), 6);
    }
}
