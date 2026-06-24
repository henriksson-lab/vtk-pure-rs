//! Vertex normal estimation methods: area-weighted, angle-weighted, PCA.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute vertex normals using area-weighted face normal averaging.
pub fn normals_area_weighted(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut nm = vec![[0.0; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        // fn_ length = 2*area, so this IS area-weighted
        for &pid in cell {
            let idx = pid as usize;
            for c in 0..3 {
                nm[idx][c] += fn_[c];
            }
        }
    }
    for n in &mut nm {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-15 {
            for c in 0..3 {
                n[c] /= len;
            }
        }
    }

    let data: Vec<f64> = nm.iter().flat_map(|n| n.iter().cloned()).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    result.point_data_mut().set_active_normals("Normals");
    result
}

/// Compute vertex normals using angle-weighted face normal averaging.
pub fn normals_angle_weighted(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut nm = vec![[0.0; 3]; n];

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let pts: Vec<[f64; 3]> = cell
            .iter()
            .map(|&pid| mesh.points.get(pid as usize))
            .collect();
        let nc = pts.len();
        let e1 = [
            pts[1][0] - pts[0][0],
            pts[1][1] - pts[0][1],
            pts[1][2] - pts[0][2],
        ];
        let e2 = [
            pts[2][0] - pts[0][0],
            pts[2][1] - pts[0][1],
            pts[2][2] - pts[0][2],
        ];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let fn_len = (fn_[0] * fn_[0] + fn_[1] * fn_[1] + fn_[2] * fn_[2]).sqrt();
        if fn_len < 1e-15 {
            continue;
        }
        let fn_n = [fn_[0] / fn_len, fn_[1] / fn_len, fn_[2] / fn_len];

        for vi in 0..nc {
            let prev = pts[(vi + nc - 1) % nc];
            let curr = pts[vi];
            let next = pts[(vi + 1) % nc];
            let ea = [next[0] - curr[0], next[1] - curr[1], next[2] - curr[2]];
            let eb = [prev[0] - curr[0], prev[1] - curr[1], prev[2] - curr[2]];
            let la = (ea[0] * ea[0] + ea[1] * ea[1] + ea[2] * ea[2]).sqrt();
            let lb = (eb[0] * eb[0] + eb[1] * eb[1] + eb[2] * eb[2]).sqrt();
            let dot = ea[0] * eb[0] + ea[1] * eb[1] + ea[2] * eb[2];
            let angle = if la * lb > 1e-15 {
                (dot / (la * lb)).clamp(-1.0, 1.0).acos()
            } else {
                0.0
            };
            let idx = cell[vi] as usize;
            for c in 0..3 {
                nm[idx][c] += angle * fn_n[c];
            }
        }
    }

    for n in &mut nm {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-15 {
            for c in 0..3 {
                n[c] /= len;
            }
        }
    }

    let data: Vec<f64> = nm.iter().flat_map(|n| n.iter().cloned()).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    result.point_data_mut().set_active_normals("Normals");
    result
}

/// Flip normals to be consistently outward-facing using a reference point.
pub fn orient_normals_outward(mesh: &PolyData, reference_point: [f64; 3]) -> PolyData {
    let n = mesh.points.len();
    let normals = match mesh.point_data().normals() {
        Some(n) => n,
        None => return normals_area_weighted(mesh),
    };
    let mut data = Vec::with_capacity(n * 3);
    let mut buf = [0.0f64; 3];

    for i in 0..n {
        normals.tuple_as_f64(i, &mut buf);
        let p = mesh.points.get(i);
        let to_ref = [
            reference_point[0] - p[0],
            reference_point[1] - p[1],
            reference_point[2] - p[2],
        ];
        let dot = buf[0] * to_ref[0] + buf[1] * to_ref[1] + buf[2] * to_ref[2];
        if dot < 0.0 {
            data.extend_from_slice(&buf);
        }
        // normal points away from ref
        else {
            data.push(-buf[0]);
            data.push(-buf[1]);
            data.push(-buf[2]);
        } // flip
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
    result.point_data_mut().set_active_normals("Normals");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn area_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = normals_area_weighted(&mesh);
        assert!(result.point_data().normals().is_some());
        let arr = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 0.01 || (buf[2] + 1.0).abs() < 0.01);
    }
    #[test]
    fn angle_normals() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = normals_angle_weighted(&mesh);
        assert!(result.point_data().normals().is_some());
    }
    #[test]
    fn orient() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh = normals_area_weighted(&mesh);
        let result = orient_normals_outward(&mesh, [0.5, 0.3, -1.0]);
        let arr = result.point_data().normals().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        // Normal should point away from reference (which is below z=0)
        assert!(buf[2] > 0.0);
    }
}
