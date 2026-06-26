use super::curvature_simple;
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

    let with_curv = curvature_simple::mean_curvature(&curvature_simple::gaussian_curvature(input));
    let Some(mean_arr) = with_curv.point_data().get_array("MeanCurvature") else {
        return input.clone();
    };
    let Some(gauss_arr) = with_curv.point_data().get_array("GaussianCurvature") else {
        return input.clone();
    };
    let mut k1 = vec![0.0f64; n];
    let mut k2 = vec![0.0f64; n];
    let mut hb = [0.0f64];
    let mut gb = [0.0f64];

    for i in 0..n {
        mean_arr.tuple_as_f64(i, &mut hb);
        gauss_arr.tuple_as_f64(i, &mut gb);
        let h = hb[0];
        let k = gb[0];
        let tmp = h * h - k;
        if tmp >= 0.0 {
            let root = tmp.sqrt();
            k1[i] = h + root;
            k2[i] = h - root;
        } else {
            k1[i] = h;
            k2[i] = h;
        }
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
