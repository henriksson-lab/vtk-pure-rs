//! Compute total angle sum around each vertex (2*pi for interior, less for boundary).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn vertex_angle_sum(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut angle_sum = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 3 {
            continue;
        }
        for i in 0..nc {
            let vi = cell[i] as usize;
            let vp = cell[(i + nc - 1) % nc] as usize;
            let vn = cell[(i + 1) % nc] as usize;
            if vi >= n || vp >= n || vn >= n {
                continue;
            }
            let p = mesh.points.get(vi);
            let pp = mesh.points.get(vp);
            let pn = mesh.points.get(vn);
            let u = [pp[0] - p[0], pp[1] - p[1], pp[2] - p[2]];
            let v = [pn[0] - p[0], pn[1] - p[1], pn[2] - p[2]];
            let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
            let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if lu > 1e-15 && lv > 1e-15 {
                let cos_a = (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (lu * lv);
                angle_sum[vi] += cos_a.clamp(-1.0, 1.0).acos();
            }
        }
    }
    let degrees: Vec<f64> = angle_sum
        .iter()
        .map(|&s| s * 180.0 / std::f64::consts::PI)
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AngleSum", degrees, 1,
        )));
    result.point_data_mut().set_active_scalars("AngleSum");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_angle_sum() {
        // Single equilateral triangle: each vertex has 60 degrees
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 3.0f64.sqrt() / 2.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = vertex_angle_sum(&mesh);
        let arr = r.point_data().get_array("AngleSum").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 60.0).abs() < 1.0);
    }
}
