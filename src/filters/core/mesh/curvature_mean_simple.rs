use crate::data::{AnyDataArray, DataArray, PolyData};

/// Estimate mean curvature at each vertex using the cotangent Laplacian.
///
/// For each vertex, computes the discrete mean curvature as half the magnitude
/// of the cotangent Laplacian vector. Adds a "MeanCurvature" point data array.
pub fn compute_mean_curvature(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    // Accumulate Laplacian vectors and area weights per vertex.
    let mut laplacian: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];
    let mut area_weight: Vec<f64> = vec![0.0; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        // Only handle triangles
        let ia: usize = cell[0] as usize;
        let ib: usize = cell[1] as usize;
        let ic: usize = cell[2] as usize;

        let a = input.points.get(ia);
        let b = input.points.get(ib);
        let c = input.points.get(ic);

        // Edge vectors
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
        let ba = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let ca = [a[0] - c[0], a[1] - c[1], a[2] - c[2]];
        let cb = [b[0] - c[0], b[1] - c[1], b[2] - c[2]];

        // Triangle area
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let tri_area: f64 =
            0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        if tri_area < 1e-30 {
            continue;
        }

        // Cotangents of each angle
        let cot_a = dot(&ab, &ac) / (2.0 * tri_area);
        let cot_b = dot(&ba, &bc) / (2.0 * tri_area);
        let cot_c = dot(&ca, &cb) / (2.0 * tri_area);

        // Cotangent Laplacian contributions: for edge (i,j) opposite angle k,
        // laplacian[i] += cot_k * (p_j - p_i)
        // Edge bc opposite A
        for d in 0..3 {
            laplacian[ib][d] += cot_a * (c[d] - b[d]);
            laplacian[ic][d] += cot_a * (b[d] - c[d]);
        }
        // Edge ac opposite B
        for d in 0..3 {
            laplacian[ia][d] += cot_b * (c[d] - a[d]);
            laplacian[ic][d] += cot_b * (a[d] - c[d]);
        }
        // Edge ab opposite C
        for d in 0..3 {
            laplacian[ia][d] += cot_c * (b[d] - a[d]);
            laplacian[ib][d] += cot_c * (a[d] - b[d]);
        }

        // Mixed area approximation: 1/3 of triangle area per vertex
        let a3: f64 = tri_area / 3.0;
        area_weight[ia] += a3;
        area_weight[ib] += a3;
        area_weight[ic] += a3;
    }

    let mut curvature: Vec<f64> = Vec::with_capacity(n);
    for i in 0..n {
        if area_weight[i] < 1e-30 {
            curvature.push(0.0);
        } else {
            let lx: f64 = laplacian[i][0] / (2.0 * area_weight[i]);
            let ly: f64 = laplacian[i][1] / (2.0 * area_weight[i]);
            let lz: f64 = laplacian[i][2] / (2.0 * area_weight[i]);
            let h: f64 = 0.5 * (lx * lx + ly * ly + lz * lz).sqrt();
            curvature.push(h);
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            curvature,
            1,
        )));
    pd
}

/// Return (min, max, average) mean curvature over all vertices.
pub fn mean_curvature_stats(input: &PolyData) -> (f64, f64, f64) {
    let result = compute_mean_curvature(input);
    let arr = result.point_data().get_array("MeanCurvature").unwrap();
    let n: usize = arr.num_tuples();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mut min_v: f64 = f64::MAX;
    let mut max_v: f64 = f64::MIN;
    let mut sum: f64 = 0.0;
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let v: f64 = buf[0];
        if v < min_v {
            min_v = v;
        }
        if v > max_v {
            max_v = v;
        }
        sum += v;
    }
    (min_v, max_v, sum / n as f64)
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_plane_zero_curvature() {
        // A larger flat mesh with an interior vertex at index 4 (center).
        // Boundary vertices will have non-zero curvature due to boundary effects.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [2.0, 0.0, 0.0], // 2
                [0.0, 1.0, 0.0], // 3
                [1.0, 1.0, 0.0], // 4 - interior
                [2.0, 1.0, 0.0], // 5
                [0.0, 2.0, 0.0], // 6
                [1.0, 2.0, 0.0], // 7
                [2.0, 2.0, 0.0], // 8
            ],
            vec![
                [0, 1, 4],
                [0, 4, 3],
                [1, 2, 5],
                [1, 5, 4],
                [3, 4, 7],
                [3, 7, 6],
                [4, 5, 8],
                [4, 8, 7],
            ],
        );
        let result = compute_mean_curvature(&pd);
        let arr = result.point_data().get_array("MeanCurvature").unwrap();
        assert_eq!(arr.num_tuples(), 9);
        // Interior vertex (index 4) on a flat plane: curvature should be ~0
        let mut val = [0.0f64];
        arr.tuple_as_f64(4, &mut val);
        assert!(
            val[0] < 0.01,
            "interior vertex curvature = {} expected ~0",
            val[0]
        );
    }

    #[test]
    fn single_triangle_curvature() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_mean_curvature(&pd);
        let arr = result.point_data().get_array("MeanCurvature").unwrap();
        assert_eq!(arr.num_tuples(), 3);
    }

    #[test]
    fn stats_returns_valid_range() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let (min_v, max_v, avg) = mean_curvature_stats(&pd);
        assert!(min_v <= avg);
        assert!(avg <= max_v);
    }
}
