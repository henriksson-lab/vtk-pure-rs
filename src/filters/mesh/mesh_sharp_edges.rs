//! Mark vertices adjacent to sharp edges (dihedral angle exceeds threshold).
use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

pub fn sharp_edge_vertices(mesh: &PolyData, angle_threshold_deg: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let threshold = angle_threshold_deg * std::f64::consts::PI / 180.0;
    let polys: Vec<Vec<usize>> = mesh
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .map(|c| c.iter().map(|&v| v as usize).collect())
        .collect();
    // Face normals
    let normals: Vec<[f64; 3]> = polys
        .iter()
        .map(|cell| polygon_normal(mesh, cell))
        .collect();
    let mut edge_faces: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (fi, cell) in polys.iter().enumerate() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            if a < n && b < n {
                let e = if a < b { (a, b) } else { (b, a) };
                edge_faces.entry(e).or_default().push(fi);
            }
        }
    }
    let mut sharp = vec![0.0f64; n];
    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = normals[faces[0]];
            let n1 = normals[faces[1]];
            let dot = n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2];
            let angle = dot.clamp(-1.0, 1.0).acos();
            if angle > threshold {
                sharp[a] = 1.0;
                sharp[b] = 1.0;
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SharpEdge",
            sharp,
            1,
        )));
    result.point_data_mut().set_active_scalars("SharpEdge");
    result
}

fn polygon_normal(mesh: &PolyData, cell: &[usize]) -> [f64; 3] {
    let n = mesh.points.len();
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;
    for i in 0..cell.len() {
        let a = cell[i];
        let b = cell[(i + 1) % cell.len()];
        if a >= n || b >= n {
            return [0.0, 0.0, 1.0];
        }
        let p = mesh.points.get(a);
        let q = mesh.points.get(b);
        nx += (p[1] - q[1]) * (p[2] + q[2]);
        ny += (p[2] - q[2]) * (p[0] + q[0]);
        nz += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-15 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CellArray, Points};

    #[test]
    fn test_sharp() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]], // 90-degree dihedral
        );
        let r = sharp_edge_vertices(&mesh, 45.0);
        let arr = r.point_data().get_array("SharpEdge").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 1.0); // vertex 0 is on sharp edge
    }

    #[test]
    fn test_sharp_quad_edge() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from_flat_vec(vec![
            0.0, 0.0, 0.0, //
            1.0, 0.0, 0.0, //
            1.0, 1.0, 0.0, //
            0.0, 1.0, 0.0, //
            1.0, 0.0, 1.0, //
            1.0, 1.0, 1.0,
        ]);
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2, 3]);
        polys.push_cell(&[1, 4, 5, 2]);
        mesh.polys = polys;

        let r = sharp_edge_vertices(&mesh, 45.0);
        let arr = r.point_data().get_array("SharpEdge").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b);
        assert_eq!(b[0], 1.0);
        arr.tuple_as_f64(2, &mut b);
        assert_eq!(b[0], 1.0);
    }
}
