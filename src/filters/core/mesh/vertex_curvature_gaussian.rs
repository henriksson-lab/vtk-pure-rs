use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate Gaussian curvature at each vertex using the angle defect method.
///
/// For each vertex, Gaussian curvature = 2*pi - sum of angles at that vertex
/// in all incident triangles. This is divided by the one-ring area (1/3 of
/// each incident triangle's area) to give the curvature per unit area.
///
/// Adds VTK's "Gauss_Curvature" point data array to the output.
pub fn vertex_curvature_gaussian(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    let mut angle_sum: Vec<f64> = vec![0.0; n];
    let mut area: Vec<f64> = vec![0.0; n];

    for cell in input.polys.iter() {
        if cell.len() == 3 {
            process_triangle(
                input,
                [cell[0], cell[1], cell[2]],
                &mut angle_sum,
                &mut area,
            );
        }
    }

    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            process_triangle(input, tri, &mut angle_sum, &mut area);
        }
    }

    let two_pi: f64 = 2.0 * std::f64::consts::PI;
    let mut curvature: Vec<f64> = vec![0.0; n];
    for i in 0..n {
        let defect: f64 = two_pi - angle_sum[i];
        if area[i] > 1e-30 {
            curvature[i] = defect / area[i];
        } else {
            curvature[i] = 0.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Gauss_Curvature",
            curvature.clone(),
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GaussianCurvature",
            curvature,
            1,
        )));
    pd.point_data_mut().set_active_scalars("Gauss_Curvature");
    pd
}

fn process_triangle(input: &PolyData, cell: [i64; 3], angle_sum: &mut [f64], area: &mut [f64]) {
    if cell[0] < 0 || cell[1] < 0 || cell[2] < 0 {
        return;
    }

    let n = input.points.len();
    let i0: usize = cell[0] as usize;
    let i1: usize = cell[1] as usize;
    let i2: usize = cell[2] as usize;
    if i0 >= n || i1 >= n || i2 >= n {
        return;
    }

    let p0 = input.points.get(i0);
    let p1 = input.points.get(i1);
    let p2 = input.points.get(i2);

    let e01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e02 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let e10 = [p0[0] - p1[0], p0[1] - p1[1], p0[2] - p1[2]];
    let e12 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
    let e20 = [p0[0] - p2[0], p0[1] - p2[1], p0[2] - p2[2]];
    let e21 = [p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2]];

    angle_sum[i0] += vec_angle(&e01, &e02);
    angle_sum[i1] += vec_angle(&e10, &e12);
    angle_sum[i2] += vec_angle(&e20, &e21);

    let cx: f64 = e01[1] * e02[2] - e01[2] * e02[1];
    let cy: f64 = e01[2] * e02[0] - e01[0] * e02[2];
    let cz: f64 = e01[0] * e02[1] - e01[1] * e02[0];
    let tri_area: f64 = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();

    let va: f64 = tri_area / 3.0;
    area[i0] += va;
    area[i1] += va;
    area[i2] += va;
}

fn vec_angle(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dot: f64 = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let la: f64 = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let lb: f64 = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    let denom: f64 = la * lb;
    if denom < 1e-30 {
        return 0.0;
    }
    let cos_val: f64 = (dot / denom).clamp(-1.0, 1.0);
    cos_val.acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_mesh_zero_curvature() {
        // A flat mesh of triangles: interior vertices should have ~0 Gaussian curvature
        // Central vertex surrounded by 6 equilateral-ish triangles
        let s: f64 = (3.0f64).sqrt() / 2.0;
        let pts = vec![
            [0.0, 0.0, 0.0],  // 0: center
            [1.0, 0.0, 0.0],  // 1
            [0.5, s, 0.0],    // 2
            [-0.5, s, 0.0],   // 3
            [-1.0, 0.0, 0.0], // 4
            [-0.5, -s, 0.0],  // 5
            [0.5, -s, 0.0],   // 6
        ];
        let tris: Vec<[i64; 3]> = vec![
            [0, 1, 2],
            [0, 2, 3],
            [0, 3, 4],
            [0, 4, 5],
            [0, 5, 6],
            [0, 6, 1],
        ];
        let pd = PolyData::from_triangles(pts, tris);
        let result = vertex_curvature_gaussian(&pd);

        let arr = result.point_data().get_array("GaussianCurvature").unwrap();
        let mut val: [f64; 1] = [0.0];
        arr.tuple_as_f64(0, &mut val); // center vertex
        assert!(
            val[0].abs() < 0.1,
            "flat center vertex curvature should be near 0, got {}",
            val[0]
        );
    }

    #[test]
    fn cube_corner_positive_curvature() {
        // Single corner of a cube: 3 right-angle triangles meeting at a vertex
        // Angle sum = 3 * pi/2, defect = 2*pi - 3*pi/2 = pi/2 > 0
        let pts = vec![
            [0.0, 0.0, 0.0], // 0: corner vertex
            [1.0, 0.0, 0.0], // 1
            [0.0, 1.0, 0.0], // 2
            [0.0, 0.0, 1.0], // 3
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2], [0, 2, 3], [0, 3, 1]];
        let pd = PolyData::from_triangles(pts, tris);
        let result = vertex_curvature_gaussian(&pd);

        let arr = result.point_data().get_array("GaussianCurvature").unwrap();
        let mut val: [f64; 1] = [0.0];
        arr.tuple_as_f64(0, &mut val); // corner vertex
        assert!(
            val[0] > 0.0,
            "cube corner should have positive Gaussian curvature, got {}",
            val[0]
        );
    }

    #[test]
    fn has_correct_array_length() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0i64, 1, 2]],
        );
        let result = vertex_curvature_gaussian(&pd);
        let arr = result.point_data().get_array("Gauss_Curvature").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        assert_eq!(arr.num_components(), 1);
        assert_eq!(
            result.point_data().scalars().unwrap().name(),
            "Gauss_Curvature"
        );
        assert!(result.point_data().get_array("GaussianCurvature").is_some());
    }

    #[test]
    fn skips_invalid_triangle_indices() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0i64, 1, 99], [-1, 1, 2]],
        );
        let result = vertex_curvature_gaussian(&pd);
        let arr = result.point_data().get_array("GaussianCurvature").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut val = [0.0; 1];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut val);
            assert_eq!(val[0], 0.0);
        }
    }
}
