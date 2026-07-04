//! Detect feature vertices (corners, edges, flat regions).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn detect_feature_vertices(mesh: &PolyData, angle_threshold: f64) -> PolyData {
    let n = mesh.points.len();
    let cos_t = angle_threshold.to_radians().cos();
    let cells = surface_cells(mesh, n);
    let fnormals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0, 0.0, 1.0];
            }
            let a = mesh.points.get(c[0]);
            let b = mesh.points.get(c[1]);
            let cc = mesh.points.get(c[2]);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
            let nn = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
            if l < 1e-15 {
                [0.0, 0.0, 1.0]
            } else {
                [nn[0] / l, nn[1] / l, nn[2] / l]
            }
        })
        .collect();
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            if a == b {
                continue;
            }
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut sharp_count = vec![0usize; n];
    for (&(a, b), faces) in &edge_faces {
        let is_feature = if faces.len() == 1 {
            true
        } else if faces.len() == 2 {
            let n1 = fnormals[faces[0]];
            let n2 = fnormals[faces[1]];
            let dot = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
            dot < cos_t
        } else {
            true
        };
        if is_feature {
            sharp_count[a] += 1;
            sharp_count[b] += 1;
        }
    }
    // Feature type: 0=flat, 1=edge, 2=corner
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if sharp_count[i] >= 3 {
                2.0
            } else if sharp_count[i] >= 1 {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "FeatureType",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("FeatureType");
    r
}

fn surface_cells(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut cells = Vec::new();
    for cell in mesh.polys.iter() {
        push_valid_cell(&mut cells, cell, n);
    }
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                push_valid_cell(&mut cells, &[tri[0], tri[1], tri[2]], n);
            } else {
                push_valid_cell(&mut cells, &[tri[1], tri[0], tri[2]], n);
            }
        }
    }
    cells
}

fn push_valid_cell(cells: &mut Vec<Vec<usize>>, cell: &[i64], n: usize) {
    let mut ids = Vec::with_capacity(cell.len());
    for &v in cell {
        let Some(v) = valid_point_index(v, n) else {
            return;
        };
        ids.push(v);
    }
    cells.push(ids);
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = detect_feature_vertices(&m, 30.0);
        assert!(r.point_data().get_array("FeatureType").is_some());
    }
}
