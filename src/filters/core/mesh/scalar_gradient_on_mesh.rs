use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the gradient of a scalar field on a triangle mesh.
///
/// Uses per-face gradient (constant per triangle from barycentric coords)
/// then averages to vertices. Adds "ScalarGradient" 3-component array
/// and "GradientMagnitude" scalar.
pub fn scalar_gradient_on_mesh(input: &PolyData, array_name: &str) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return input.clone(),
    };
    if n == 0 {
        return input.clone();
    }

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut vgrad = vec![[0.0f64; 3]; n];
    let mut vcount = vec![0usize; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let i0 = cell[0] as usize;
        let i1 = cell[1] as usize;
        let i2 = cell[2] as usize;
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let v0 = input.points.get(i0);
        let v1 = input.points.get(i1);
        let v2 = input.points.get(i2);

        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let normal = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let area2 = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
        if area2 < 1e-20 {
            continue;
        }

        // Gradient = (1/2A) * sum(f_i * (n × e_opp))
        let f0 = values[i0];
        let f1 = values[i1];
        let f2 = values[i2];

        // Edge opposite to each vertex
        let e_opp0 = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]]; // opposite v0
        let e_opp1 = [v0[0] - v2[0], v0[1] - v2[1], v0[2] - v2[2]]; // opposite v1
        let e_opp2 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]]; // opposite v2

        let cross_n_e = |e: [f64; 3]| {
            [
                normal[1] * e[2] - normal[2] * e[1],
                normal[2] * e[0] - normal[0] * e[2],
                normal[0] * e[1] - normal[1] * e[0],
            ]
        };

        let g0 = cross_n_e(e_opp0);
        let g1 = cross_n_e(e_opp1);
        let g2 = cross_n_e(e_opp2);

        let grad = [
            (f0 * g0[0] + f1 * g1[0] + f2 * g2[0]) / area2,
            (f0 * g0[1] + f1 * g1[1] + f2 * g2[1]) / area2,
            (f0 * g0[2] + f1 * g1[2] + f2 * g2[2]) / area2,
        ];

        for &id in &[i0, i1, i2] {
            vgrad[id][0] += grad[0];
            vgrad[id][1] += grad[1];
            vgrad[id][2] += grad[2];
            vcount[id] += 1;
        }
    }

    // Average
    for i in 0..n {
        if vcount[i] > 0 {
            let c = vcount[i] as f64;
            vgrad[i][0] /= c;
            vgrad[i][1] /= c;
            vgrad[i][2] /= c;
        }
    }

    let flat: Vec<f64> = vgrad.iter().flat_map(|g| g.iter().copied()).collect();
    let mag: Vec<f64> = vgrad
        .iter()
        .map(|g| (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt())
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ScalarGradient",
            flat,
            3,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            mag,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_gradient() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.0],
                1,
            )));

        let result = scalar_gradient_on_mesh(&pd, "f");
        assert!(result.point_data().get_array("ScalarGradient").is_some());
        let mag = result.point_data().get_array("GradientMagnitude").unwrap();
        let mut buf = [0.0f64];
        mag.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.5); // gradient should be ~1 in X direction
    }

    #[test]
    fn constant_zero_gradient() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vec![5.0; 3], 1)));

        let result = scalar_gradient_on_mesh(&pd, "f");
        let mag = result.point_data().get_array("GradientMagnitude").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            mag.tuple_as_f64(i, &mut buf);
            assert!(buf[0] < 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = scalar_gradient_on_mesh(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
