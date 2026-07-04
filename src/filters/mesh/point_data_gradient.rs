use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the gradient of a scalar field on a triangle mesh.
///
/// Uses a constant per-face gradient for each triangle fan, then averages
/// incident face gradients to vertices.
pub fn scalar_gradient_on_mesh(input: &PolyData, array_name: &str) -> PolyData {
    let n: usize = input.points.len();
    let values: Vec<f64> = match input.point_data().get_array(array_name) {
        Some(arr) if arr.num_tuples() >= n => {
            let mut vals = vec![0.0f64; n];
            let mut buf = [0.0f64];
            for (i, val) in vals.iter_mut().enumerate() {
                arr.tuple_as_f64(i, &mut buf);
                *val = buf[0];
            }
            vals
        }
        _ => return input.clone(),
    };

    if n == 0 {
        return input.clone();
    }

    let mut vertex_gradients = vec![[0.0f64; 3]; n];
    let mut vertex_counts = vec![0usize; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(i0) = valid_point_id(cell[0], n) else {
            continue;
        };
        for k in 1..cell.len() - 1 {
            let Some(i1) = valid_point_id(cell[k], n) else {
                continue;
            };
            let Some(i2) = valid_point_id(cell[k + 1], n) else {
                continue;
            };
            if let Some(gradient) = triangle_gradient(input, &values, [i0, i1, i2]) {
                for id in [i0, i1, i2] {
                    vertex_gradients[id][0] += gradient[0];
                    vertex_gradients[id][1] += gradient[1];
                    vertex_gradients[id][2] += gradient[2];
                    vertex_counts[id] += 1;
                }
            }
        }
    }

    for (gradient, count) in vertex_gradients.iter_mut().zip(vertex_counts) {
        if count > 0 {
            let denom = count as f64;
            gradient[0] /= denom;
            gradient[1] /= denom;
            gradient[2] /= denom;
        }
    }

    let grad_data: Vec<f64> = vertex_gradients
        .iter()
        .flat_map(|gradient| gradient.iter().copied())
        .collect();
    let magnitudes: Vec<f64> = vertex_gradients
        .iter()
        .map(|gradient| {
            (gradient[0] * gradient[0] + gradient[1] * gradient[1] + gradient[2] * gradient[2])
                .sqrt()
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ScalarGradient",
            grad_data,
            3,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            magnitudes,
            1,
        )));
    pd
}

fn valid_point_id(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn triangle_gradient(input: &PolyData, values: &[f64], ids: [usize; 3]) -> Option<[f64; 3]> {
    let [i0, i1, i2] = ids;
    let v0 = input.points.get(i0);
    let v1 = input.points.get(i1);
    let v2 = input.points.get(i2);

    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let normal = cross(e1, e2);
    let area2 = dot(normal, normal);
    if area2 < 1e-20 {
        return None;
    }

    let e_opp0 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let e_opp1 = [v0[0] - v2[0], v0[1] - v2[1], v0[2] - v2[2]];
    let e_opp2 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];

    let g0 = cross(normal, e_opp0);
    let g1 = cross(normal, e_opp1);
    let g2 = cross(normal, e_opp2);
    let f0 = values[i0];
    let f1 = values[i1];
    let f2 = values[i2];

    Some([
        (f0 * g0[0] + f1 * g1[0] + f2 * g2[0]) / area2,
        (f0 * g0[1] + f1 * g1[1] + f2 * g2[1]) / area2,
        (f0 * g0[2] + f1 * g1[2] + f2 * g2[2]) / area2,
    ])
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_of_linear_x_field() {
        // f(x,y,z) = x => gradient should be (1, 0, 0)
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [1.0, 2.0, 0.0],
            ],
            vec![[0, 1, 3], [1, 2, 4], [1, 4, 3], [3, 4, 5]],
        );
        let scalars = DataArray::from_vec("height", vec![0.0, 1.0, 2.0, 0.5, 1.5, 1.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = scalar_gradient_on_mesh(&pd, "height");
        let g = result.point_data().get_array("ScalarGradient").unwrap();
        assert_eq!(g.num_tuples(), 6);
        assert_eq!(g.num_components(), 3);

        let mut val = [0.0f64; 3];
        let mut found: bool = false;
        for vi in 0..6 {
            g.tuple_as_f64(vi, &mut val);
            if (val[0] - 1.0).abs() < 0.5 && val[1].abs() < 0.5 {
                found = true;
                break;
            }
        }
        assert!(found, "expected at least one vertex with gradient ~(1,0,0)");
    }

    #[test]
    fn gradient_missing_array_returns_clone() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = scalar_gradient_on_mesh(&pd, "nonexistent");
        assert_eq!(result.points.len(), 3);
        assert!(result.point_data().get_array("ScalarGradient").is_none());
    }

    #[test]
    fn gradient_of_linear_y_field() {
        // f(x,y,z) = y => gradient should be (0, 1, 0)
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [1.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [2, 3, 4]],
        );
        let scalars = DataArray::from_vec("f", vec![0.0, 0.0, 1.0, 1.0, 2.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = scalar_gradient_on_mesh(&pd, "f");
        let g = result.point_data().get_array("ScalarGradient").unwrap();
        let mut val = [0.0f64; 3];
        let mut found: bool = false;
        for vi in 0..5 {
            g.tuple_as_f64(vi, &mut val);
            if val[0].abs() < 0.5 && (val[1] - 1.0).abs() < 0.5 {
                found = true;
                break;
            }
        }
        assert!(found, "expected at least one vertex with gradient ~(0,1,0)");
    }
}
