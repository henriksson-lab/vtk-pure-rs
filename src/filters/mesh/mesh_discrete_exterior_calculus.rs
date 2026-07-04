//! Discrete Exterior Calculus (DEC) operators on mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn dec_gradient(mesh: &PolyData, array_name: &str) -> PolyData {
    // Gradient of 0-form (scalar) to 1-form (per-edge), projected to vertex vectors
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    if arr.num_tuples() != n {
        return mesh.clone();
    }
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut grad = vec![[0.0f64; 3]; n];
    let mut wt = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let Some(ids) = triangle_point_ids(cell, n) else {
            continue;
        };
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let nm = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let a2 = nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2];
        if a2 < 1e-30 {
            continue;
        }
        // Gradient in triangle plane, normalized by 2*area.
        let df1 = vals[ids[1]] - vals[ids[0]];
        let df2 = vals[ids[2]] - vals[ids[0]];
        // perp(e) = n x e / |n|
        let pe2 = [
            (nm[1] * e2[2] - nm[2] * e2[1]) / a2,
            (nm[2] * e2[0] - nm[0] * e2[2]) / a2,
            (nm[0] * e2[1] - nm[1] * e2[0]) / a2,
        ];
        let pe1 = [
            (nm[1] * e1[2] - nm[2] * e1[1]) / a2,
            (nm[2] * e1[0] - nm[0] * e1[2]) / a2,
            (nm[0] * e1[1] - nm[1] * e1[0]) / a2,
        ];
        let g = [
            df2 * pe1[0] - df1 * pe2[0],
            df2 * pe1[1] - df1 * pe2[1],
            df2 * pe1[2] - df1 * pe2[2],
        ];
        for &vi in &ids {
            grad[vi][0] += g[0];
            grad[vi][1] += g[1];
            grad[vi][2] += g[2];
            wt[vi] += 1.0;
        }
    }
    for i in 0..n {
        if wt[i] > 0.0 {
            grad[i][0] /= wt[i];
            grad[i][1] /= wt[i];
            grad[i][2] /= wt[i];
        }
    }
    let data: Vec<f64> = grad.iter().flat_map(|g| g.iter().copied()).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DECGradient",
            data,
            3,
        )));
    r
}
pub fn dec_divergence(mesh: &PolyData, vector_array: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(vector_array) {
        Some(a) if a.num_components() == 3 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    if arr.num_tuples() != n {
        return mesh.clone();
    }
    let mut buf = [0.0f64; 3];
    let vecs: Vec<[f64; 3]> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1], buf[2]]
        })
        .collect();
    let mut div = vec![0.0f64; n];
    let mut area = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let Some(ids) = triangle_point_ids(cell, n) else {
            continue;
        };
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
        let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
        let tri_area = 0.5
            * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
        let grad_phi = [
            triangle_gradient(p, [1.0, 0.0, 0.0]),
            triangle_gradient(p, [0.0, 1.0, 0.0]),
            triangle_gradient(p, [0.0, 0.0, 1.0]),
        ];
        let tri_div = vecs[ids[0]][0] * grad_phi[0][0]
            + vecs[ids[0]][1] * grad_phi[0][1]
            + vecs[ids[0]][2] * grad_phi[0][2]
            + vecs[ids[1]][0] * grad_phi[1][0]
            + vecs[ids[1]][1] * grad_phi[1][1]
            + vecs[ids[1]][2] * grad_phi[1][2]
            + vecs[ids[2]][0] * grad_phi[2][0]
            + vecs[ids[2]][1] * grad_phi[2][1]
            + vecs[ids[2]][2] * grad_phi[2][2];
        for &vi in &ids {
            div[vi] += tri_div * tri_area / 3.0;
            area[vi] += tri_area / 3.0;
        }
    }
    for i in 0..n {
        if area[i] > 1e-15 {
            div[i] /= area[i];
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DECDivergence",
            div,
            1,
        )));
    r
}

fn triangle_gradient(p: [[f64; 3]; 3], values: [f64; 3]) -> [f64; 3] {
    let e1 = [p[1][0] - p[0][0], p[1][1] - p[0][1], p[1][2] - p[0][2]];
    let e2 = [p[2][0] - p[0][0], p[2][1] - p[0][1], p[2][2] - p[0][2]];
    let nm = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let a2 = nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2];
    if a2 < 1e-30 {
        return [0.0; 3];
    }

    let df1 = values[1] - values[0];
    let df2 = values[2] - values[0];
    let pe2 = [
        (nm[1] * e2[2] - nm[2] * e2[1]) / a2,
        (nm[2] * e2[0] - nm[0] * e2[2]) / a2,
        (nm[0] * e2[1] - nm[1] * e2[0]) / a2,
    ];
    let pe1 = [
        (nm[1] * e1[2] - nm[2] * e1[1]) / a2,
        (nm[2] * e1[0] - nm[0] * e1[2]) / a2,
        (nm[0] * e1[1] - nm[1] * e1[0]) / a2,
    ];
    [
        df2 * pe1[0] - df1 * pe2[0],
        df2 * pe1[1] - df1 * pe2[1],
        df2 * pe1[2] - df1 * pe2[2],
    ]
}

fn triangle_point_ids(cell: &[i64], num_points: usize) -> Option<[usize; 3]> {
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
    fn test_grad() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 2.0, 1.0],
                1,
            )));
        let r = dec_gradient(&m, "s");
        assert!(r.point_data().get_array("DECGradient").is_some());
        assert_eq!(
            r.point_data()
                .get_array("DECGradient")
                .unwrap()
                .num_components(),
            3
        );
        let arr = r.point_data().get_array("DECGradient").unwrap();
        let mut g = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut g);
        assert!((g[0] - 1.0).abs() < 1e-9);
        assert!(g[1].abs() < 1e-9);
        assert!(g[2].abs() < 1e-9);
    }
    #[test]
    fn test_div() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));
        let r = dec_divergence(&m, "v");
        assert!(r.point_data().get_array("DECDivergence").is_some());
        let arr = r.point_data().get_array("DECDivergence").unwrap();
        for value in arr.to_f64_vec() {
            assert!(value.abs() < 1e-12);
        }
    }
}
