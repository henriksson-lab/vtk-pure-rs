//! Estimate scalar-field gradient on mesh vertices.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn scalar_gradient(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        None => return mesh.clone(),
        _ => return mesh.clone(),
    };
    if n == 0 {
        return mesh.clone();
    }
    let mut vals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals[i] = buf[0];
    }
    let mut grad = vec![[0.0f64; 3]; n];
    let mut weight = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let Some(ids) = valid_triangle_ids(cell, n) else {
            continue;
        };
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let normal = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let area2 = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
        if area2 < 1e-30 {
            continue;
        }
        let e12 = [p[2][0] - p[1][0], p[2][1] - p[1][1], p[2][2] - p[1][2]];
        let e20 = [p[0][0] - p[2][0], p[0][1] - p[2][1], p[0][2] - p[2][2]];
        let cross = |a: [f64; 3], b: [f64; 3]| -> [f64; 3] {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let c0 = cross(normal, e12);
        let c1 = cross(normal, e20);
        let c2 = cross(normal, e1);
        let cell_grad = [
            (vals[ids[0]] * c0[0] + vals[ids[1]] * c1[0] + vals[ids[2]] * c2[0]) / area2,
            (vals[ids[0]] * c0[1] + vals[ids[1]] * c1[1] + vals[ids[2]] * c2[1]) / area2,
            (vals[ids[0]] * c0[2] + vals[ids[1]] * c1[2] + vals[ids[2]] * c2[2]) / area2,
        ];
        for &id in &ids {
            grad[id][0] += cell_grad[0];
            grad[id][1] += cell_grad[1];
            grad[id][2] += cell_grad[2];
            weight[id] += 1.0;
        }
    }
    for i in 0..n {
        if weight[i] > 0.0 {
            grad[i][0] /= weight[i];
            grad[i][1] /= weight[i];
            grad[i][2] /= weight[i];
        }
    }
    let gradient: Vec<f64> = grad.iter().flat_map(|g| g.iter().copied()).collect();
    let grad_mag: Vec<f64> = grad
        .iter()
        .map(|g| (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt())
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ScalarGradient",
            gradient,
            3,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            grad_mag,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("GradientMagnitude");
    result
}

fn valid_triangle_ids(cell: &[i64], num_points: usize) -> Option<[usize; 3]> {
    Some([
        valid_point_id(cell[0], num_points)?,
        valid_point_id(cell[1], num_points)?,
        valid_point_id(cell[2], num_points)?,
    ])
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gradient() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Linear scalar: gradient should be ~1
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = scalar_gradient(&mesh, "f");
        let arr = r.point_data().get_array("GradientMagnitude").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!(b[0] > 0.5);
    }
}
