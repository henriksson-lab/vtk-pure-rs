use std::collections::{HashMap, HashSet};

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute gradient of a scalar field defined on mesh points using least-squares
/// fitting in the 1-ring neighborhood of each vertex.
///
/// Adds a 3-component "ScalarGradient" array to the output point data.
/// If the named array is not found, returns a clone of the input.
pub fn scalar_gradient_on_mesh(input: &PolyData, array_name: &str) -> PolyData {
    let n: usize = input.points.len();
    let scalars: Vec<f64> = match input.point_data().get_array(array_name) {
        Some(arr) => {
            let mut vals = vec![0.0f64; n];
            let mut buf = [0.0f64];
            for (i, val) in vals.iter_mut().enumerate() {
                arr.tuple_as_f64(i, &mut buf);
                *val = buf[0];
            }
            vals
        }
        None => return input.clone(),
    };

    // Build 1-ring adjacency from polygon cells
    let mut neighbor_sets: HashMap<usize, HashSet<usize>> = HashMap::new();
    for cell in input.polys.iter() {
        let nc: usize = cell.len();
        for i in 0..nc {
            let a: usize = cell[i] as usize;
            let b: usize = cell[(i + 1) % nc] as usize;
            neighbor_sets.entry(a).or_default().insert(b);
            neighbor_sets.entry(b).or_default().insert(a);
        }
    }
    let neighbors: HashMap<usize, Vec<usize>> = neighbor_sets
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().collect()))
        .collect();

    let mut grad_data = vec![0.0f64; n * 3];

    for i in 0..n {
        let pi = input.points.get(i);
        let si: f64 = scalars[i];

        let nbrs = match neighbors.get(&i) {
            Some(nb) => nb,
            None => continue,
        };

        // Least-squares gradient: minimize sum_j |grad . (pj - pi) - (sj - si)|^2
        // Normal equations: A^T A x = A^T b
        let mut ata = [[0.0f64; 3]; 3];
        let mut atb = [0.0f64; 3];

        for &j in nbrs {
            let pj = input.points.get(j);
            let dx: [f64; 3] = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
            let ds: f64 = scalars[j] - si;

            for r in 0..3 {
                for c in 0..3 {
                    ata[r][c] += dx[r] * dx[c];
                }
                atb[r] += dx[r] * ds;
            }
        }

        if let Some(g) = solve_3x3(&ata, &atb) {
            grad_data[i * 3] = g[0];
            grad_data[i * 3 + 1] = g[1];
            grad_data[i * 3 + 2] = g[2];
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ScalarGradient",
            grad_data,
            3,
        )));
    pd
}

fn solve_3x3(a: &[[f64; 3]; 3], b: &[f64; 3]) -> Option<[f64; 3]> {
    let eps: f64 = 1e-10;
    let ar = [
        [a[0][0] + eps, a[0][1], a[0][2]],
        [a[1][0], a[1][1] + eps, a[1][2]],
        [a[2][0], a[2][1], a[2][2] + eps],
    ];

    let det: f64 = ar[0][0] * (ar[1][1] * ar[2][2] - ar[1][2] * ar[2][1])
        - ar[0][1] * (ar[1][0] * ar[2][2] - ar[1][2] * ar[2][0])
        + ar[0][2] * (ar[1][0] * ar[2][1] - ar[1][1] * ar[2][0]);

    if det.abs() < 1e-40 {
        return None;
    }

    let inv_det: f64 = 1.0 / det;

    let x: f64 = (b[0] * (ar[1][1] * ar[2][2] - ar[1][2] * ar[2][1])
        - ar[0][1] * (b[1] * ar[2][2] - ar[1][2] * b[2])
        + ar[0][2] * (b[1] * ar[2][1] - ar[1][1] * b[2]))
        * inv_det;

    let y: f64 = (ar[0][0] * (b[1] * ar[2][2] - ar[1][2] * b[2])
        - b[0] * (ar[1][0] * ar[2][2] - ar[1][2] * ar[2][0])
        + ar[0][2] * (ar[1][0] * b[2] - b[1] * ar[2][0]))
        * inv_det;

    let z: f64 = (ar[0][0] * (ar[1][1] * b[2] - b[1] * ar[2][1])
        - ar[0][1] * (ar[1][0] * b[2] - b[1] * ar[2][0])
        + b[0] * (ar[1][0] * ar[2][1] - ar[1][1] * ar[2][0]))
        * inv_det;

    Some([x, y, z])
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
