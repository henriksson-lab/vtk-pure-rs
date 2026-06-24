use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute principal curvatures (k1, k2) at each vertex.
///
/// Uses the shape operator approximation: for each vertex, fits a
/// quadratic to the one-ring neighborhood projected onto the tangent plane.
/// Adds "K1" (max curvature), "K2" (min curvature), and "ShapeIndex" arrays.
pub fn principal_curvatures(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut k1 = vec![0.0f64; n];
    let mut k2 = vec![0.0f64; n];

    for i in 0..n {
        if neighbors[i].len() < 3 {
            continue;
        }

        // Estimate normal as average of edge cross products
        let p = pts[i];
        let nbrs = &neighbors[i];
        let nn = nbrs.len();
        let mut normal = [0.0; 3];
        for j in 0..nn {
            let a = pts[nbrs[j]];
            let b = pts[nbrs[(j + 1) % nn]];
            let e1 = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
            let e2 = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
            normal[0] += e1[1] * e2[2] - e1[2] * e2[1];
            normal[1] += e1[2] * e2[0] - e1[0] * e2[2];
            normal[2] += e1[0] * e2[1] - e1[1] * e2[0];
        }
        let nlen = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if nlen < 1e-15 {
            continue;
        }
        normal[0] /= nlen;
        normal[1] /= nlen;
        normal[2] /= nlen;

        // Estimate curvatures from neighbor heights above tangent plane
        let mut sum_h = 0.0;
        let mut sum_h2 = 0.0;
        let mut sum_r2 = 0.0;
        for &j in nbrs {
            let d = [pts[j][0] - p[0], pts[j][1] - p[1], pts[j][2] - p[2]];
            let h = d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2]; // height
            let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2] - h * h; // tangent distance^2
            sum_h += h;
            sum_h2 += h * h;
            sum_r2 += r2.max(1e-15);
        }
        let mean_curv = sum_h / sum_r2.max(1e-15) * 2.0;
        let gauss_curv = (sum_h2 - sum_h * sum_h / nn as f64) / sum_r2.max(1e-15);

        // k1, k2 from mean and Gaussian curvature
        let disc = (mean_curv * mean_curv - gauss_curv).max(0.0).sqrt();
        k1[i] = mean_curv + disc;
        k2[i] = mean_curv - disc;
    }

    // Shape index: (2/π) * atan((k1+k2)/(k1-k2))
    let shape_index: Vec<f64> = (0..n)
        .map(|i| {
            let diff = k1[i] - k2[i];
            if diff.abs() > 1e-15 {
                (2.0 / std::f64::consts::PI) * ((k1[i] + k2[i]) / diff).atan()
            } else {
                0.0
            }
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("K1", k1, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("K2", k2, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeIndex",
            shape_index,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_curvature_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.5]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = principal_curvatures(&pd);
        assert!(result.point_data().get_array("K1").is_some());
        assert!(result.point_data().get_array("K2").is_some());
        assert!(result.point_data().get_array("ShapeIndex").is_some());
    }

    #[test]
    fn flat_surface_zero() {
        let mut pd = PolyData::new();
        // Regular grid -> flat -> zero curvature
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = j * 3 + i;
                let b = a + 1;
                let c = a + 4;
                let d = a + 3;
                pd.polys.push_cell(&[a as i64, b as i64, c as i64]);
                pd.polys.push_cell(&[a as i64, c as i64, d as i64]);
            }
        }

        let result = principal_curvatures(&pd);
        let arr = result.point_data().get_array("K1").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf); // center vertex
        assert!(buf[0].abs() < 0.5); // approximately flat
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = principal_curvatures(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
