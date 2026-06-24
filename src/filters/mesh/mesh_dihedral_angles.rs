//! Compute dihedral angles at each edge and store max per vertex.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn dihedral_angles(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let normals: Vec<[f64; 3]> = tris
        .iter()
        .map(|&[a, b, c]| {
            if a >= n || b >= n || c >= n {
                return [0.0, 0.0, 1.0];
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let nx = u[1] * v[2] - u[2] * v[1];
            let ny = u[2] * v[0] - u[0] * v[2];
            let nz = u[0] * v[1] - u[1] * v[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len > 1e-15 {
                [nx / len, ny / len, nz / len]
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect();
    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, &[a, b, c]) in tris.iter().enumerate() {
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
            edge_faces.entry(e).or_default().push(fi);
        }
    }
    let mut max_angle = vec![0.0f64; n];
    for (&(a, b), faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = normals[faces[0]];
            let n1 = normals[faces[1]];
            let dot = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
            let angle = dot.acos() * 180.0 / std::f64::consts::PI;
            if angle > max_angle[a] {
                max_angle[a] = angle;
            }
            if angle > max_angle[b] {
                max_angle[b] = angle;
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MaxDihedralAngle",
            max_angle,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("MaxDihedralAngle");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dihedral() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let r = dihedral_angles(&mesh);
        let arr = r.point_data().get_array("MaxDihedralAngle").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] > 80.0); // ~90 degrees
    }
}
