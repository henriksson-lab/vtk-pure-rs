use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate principal curvatures (k1, k2) at each vertex from mean and
/// Gaussian curvature.
///
/// Uses the angle-deficit method for Gaussian curvature and cotangent-weight
/// formula for mean curvature, then derives:
///   k1 = H + sqrt(H^2 - K)
///   k2 = H - sqrt(H^2 - K)
///
/// Adds "PrincipalCurvature1" and "PrincipalCurvature2" point data arrays.
/// Input should be a triangle mesh.
pub fn compute_principal_curvatures(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut gauss = vec![0.0f64; n];
    let mut mean = vec![0.0f64; n];
    let mut area = vec![0.0f64; n];

    // Initialize Gaussian curvature with 2*pi (angle deficit starts from full circle)
    gauss.fill(2.0 * std::f64::consts::PI);

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let i0 = cell[0] as usize;
        let i1 = cell[1] as usize;
        let i2 = cell[2] as usize;

        let p0 = input.points.get(i0);
        let p1 = input.points.get(i1);
        let p2 = input.points.get(i2);

        let e01 = sub(p1, p0);
        let e02 = sub(p2, p0);
        let e12 = sub(p2, p1);
        let e10 = sub(p0, p1);
        let e20 = sub(p0, p2);
        let e21 = sub(p1, p2);

        let a0 = angle_between(e01, e02);
        let a1 = angle_between(e10, e12);
        let a2 = angle_between(e20, e21);

        gauss[i0] -= a0;
        gauss[i1] -= a1;
        gauss[i2] -= a2;

        let cross = cross_3(e01, e02);
        let tri_area: f64 = 0.5 * length(cross);

        let va: f64 = tri_area / 3.0;
        area[i0] += va;
        area[i1] += va;
        area[i2] += va;

        let cot0 = cot(a0);
        let cot1 = cot(a1);
        let cot2 = cot(a2);

        let len_e12: f64 = length(e12);
        let len_e02: f64 = length(e02);
        let len_e01: f64 = length(e01);

        mean[i0] += cot1 * len_e02 * len_e02 + cot2 * len_e01 * len_e01;
        mean[i1] += cot0 * len_e12 * len_e12 + cot2 * len_e01 * len_e01;
        mean[i2] += cot0 * len_e12 * len_e12 + cot1 * len_e02 * len_e02;
    }

    // Normalize by area and compute principal curvatures
    let mut k1 = vec![0.0f64; n];
    let mut k2 = vec![0.0f64; n];

    for i in 0..n {
        if area[i] > 1e-20 {
            let g: f64 = gauss[i] / area[i];
            let h: f64 = mean[i] / (4.0 * area[i]);

            let disc: f64 = (h * h - g).max(0.0);
            let sqrt_disc: f64 = disc.sqrt();
            k1[i] = h + sqrt_disc;
            k2[i] = h - sqrt_disc;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PrincipalCurvature1",
            k1,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PrincipalCurvature2",
            k2,
            1,
        )));
    pd
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross_3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let la: f64 = length(a);
    let lb: f64 = length(b);
    if la < 1e-20 || lb < 1e-20 {
        return 0.0;
    }
    let cos_angle: f64 = (dot(a, b) / (la * lb)).clamp(-1.0, 1.0);
    cos_angle.acos()
}

fn cot(angle: f64) -> f64 {
    let s: f64 = angle.sin();
    if s.abs() < 1e-20 {
        0.0
    } else {
        angle.cos() / s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrays_present_and_sized() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_principal_curvatures(&pd);
        let k1 = result
            .point_data()
            .get_array("PrincipalCurvature1")
            .unwrap();
        let k2 = result
            .point_data()
            .get_array("PrincipalCurvature2")
            .unwrap();
        assert_eq!(k1.num_tuples(), 3);
        assert_eq!(k2.num_tuples(), 3);
    }

    #[test]
    fn flat_mesh_k1_ge_k2() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 4], [0, 4, 3], [1, 2, 5], [1, 5, 4]],
        );
        let result = compute_principal_curvatures(&pd);
        let k1_arr = result
            .point_data()
            .get_array("PrincipalCurvature1")
            .unwrap();
        let k2_arr = result
            .point_data()
            .get_array("PrincipalCurvature2")
            .unwrap();
        let mut v1 = [0.0f64];
        let mut v2 = [0.0f64];
        for i in 0..6 {
            k1_arr.tuple_as_f64(i, &mut v1);
            k2_arr.tuple_as_f64(i, &mut v2);
            assert!(v1[0] >= v2[0] - 1e-10, "k1 should be >= k2 at vertex {}", i);
        }
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::from_triangles(vec![], vec![]);
        let result = compute_principal_curvatures(&pd);
        let k1 = result
            .point_data()
            .get_array("PrincipalCurvature1")
            .unwrap();
        assert_eq!(k1.num_tuples(), 0);
    }
}
