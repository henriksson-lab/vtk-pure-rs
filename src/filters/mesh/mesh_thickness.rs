//! Estimate local thickness of a closed mesh by ray casting inward.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn thickness_analysis(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        for &vi in &cell[..] {
            let vi = vi as usize;
            if vi < n {
                vnormals[vi][0] += nx;
                vnormals[vi][1] += ny;
                vnormals[vi][2] += nz;
            }
        }
    }
    for vn in &mut vnormals {
        let len = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if len > 1e-15 {
            vn[0] /= len;
            vn[1] /= len;
            vn[2] /= len;
        }
    }
    // Cast ray inward (negative normal) and find closest intersection
    let mut thickness = vec![0.0f64; n];
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [-vnormals[i][0], -vnormals[i][1], -vnormals[i][2]];
        let mut min_t = f64::INFINITY;
        for &[a, b, c] in &tris {
            if a == i || b == i || c == i {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            // Möller–Trumbore intersection
            let e1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let e2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let h = [
                d[1] * e2[2] - d[2] * e2[1],
                d[2] * e2[0] - d[0] * e2[2],
                d[0] * e2[1] - d[1] * e2[0],
            ];
            let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
            if det.abs() < 1e-12 {
                continue;
            }
            let inv_det = 1.0 / det;
            let s = [p[0] - pa[0], p[1] - pa[1], p[2] - pa[2]];
            let u = inv_det * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
            if u < 0.0 || u > 1.0 {
                continue;
            }
            let q = [
                s[1] * e1[2] - s[2] * e1[1],
                s[2] * e1[0] - s[0] * e1[2],
                s[0] * e1[1] - s[1] * e1[0],
            ];
            let v = inv_det * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = inv_det * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
            if t > 0.01 && t < min_t {
                min_t = t;
            }
        }
        thickness[i] = if min_t.is_finite() { min_t } else { 0.0 };
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Thickness",
            thickness,
            1,
        )));
    result.point_data_mut().set_active_scalars("Thickness");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_thickness() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.1],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = thickness_analysis(&mesh);
        assert!(r.point_data().get_array("Thickness").is_some());
    }
}
