use crate::data::PolyData;

/// Result of integrating attributes over a mesh.
#[derive(Debug, Clone)]
pub struct IntegrationResult {
    /// Total surface area.
    pub area: f64,
    /// Per-array integrated values. Each entry is (name, integrated value per component).
    pub integrated: Vec<(String, Vec<f64>)>,
}

/// Integrate point data arrays over the surface of a PolyData.
///
/// For each scalar/vector array in point data, computes the integral
/// over the surface by summing the product of the interpolated value
/// at each triangle's centroid times the triangle's area.
pub fn integrate_attributes(input: &PolyData) -> IntegrationResult {
    let mut total_area = 0.0f64;

    // For each array, accumulate weighted sum
    let n_arrays = input.point_data().num_arrays();
    let mut sums: Vec<Vec<f64>> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    for ai in 0..n_arrays {
        if let Some(arr) = input.point_data().get_array_by_index(ai) {
            names.push(arr.name().to_string());
            sums.push(vec![0.0; arr.num_components()]);
        }
    }

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let p0 = input.points.get(cell[0] as usize);

        for i in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);

            // Triangle area
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            total_area += area;

            // Average value at triangle centroid = (v0 + v1 + v2) / 3
            let tri_pts = [cell[0] as usize, cell[i] as usize, cell[i + 1] as usize];

            for (ai, sum_vec) in sums.iter_mut().enumerate() {
                if let Some(arr) = input.point_data().get_array_by_index(ai) {
                    let nc = arr.num_components();
                    let mut buf = vec![0.0f64; nc];
                    for &pi in &tri_pts {
                        arr.tuple_as_f64(pi, &mut buf);
                        for (c, s) in sum_vec.iter_mut().enumerate() {
                            *s += buf[c] * area / 3.0;
                        }
                    }
                }
            }
        }
    }

    IntegrationResult {
        area: total_area,
        integrated: names.into_iter().zip(sums).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn integrate_constant_scalar() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Constant scalar = 2.0
        let scalars = DataArray::from_vec("density", vec![2.0, 2.0, 2.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = integrate_attributes(&pd);
        // Area of triangle = 0.5
        assert!((result.area - 0.5).abs() < 1e-10);
        // Integral of constant 2.0 over area 0.5 = 1.0
        assert_eq!(result.integrated.len(), 1);
        assert!((result.integrated[0].1[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn integrate_no_data() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = integrate_attributes(&pd);
        assert!((result.area - 2.0).abs() < 1e-10);
        assert!(result.integrated.is_empty());
    }
}
