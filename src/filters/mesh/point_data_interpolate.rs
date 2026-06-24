use crate::data::{AnyDataArray, DataArray, PolyData};

/// Interpolate a scalar array from one mesh to another using nearest-neighbor lookup.
///
/// For each point in `target`, finds the closest point in `input` (brute force)
/// and copies the value of `array_name` from that closest point. The result is
/// a clone of `target` with the interpolated array added to point data.
pub fn interpolate_nearest(input: &PolyData, target: &PolyData, array_name: &str) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return target.clone(),
    };

    let nc: usize = arr.num_components();
    let src_n: usize = input.points.len();
    let tgt_n: usize = target.points.len();

    let mut out_data: Vec<f64> = Vec::with_capacity(tgt_n * nc);
    let mut buf: Vec<f64> = vec![0.0; nc];

    for ti in 0..tgt_n {
        let tp = target.points.get(ti);

        // Brute force nearest neighbor.
        let mut best_dist_sq: f64 = f64::MAX;
        let mut best_idx: usize = 0;

        for si in 0..src_n {
            let sp = input.points.get(si);
            let dx: f64 = tp[0] - sp[0];
            let dy: f64 = tp[1] - sp[1];
            let dz: f64 = tp[2] - sp[2];
            let dist_sq: f64 = dx * dx + dy * dy + dz * dz;
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_idx = si;
            }
        }

        arr.tuple_as_f64(best_idx, &mut buf);
        out_data.extend_from_slice(&buf);
    }

    let mut result = target.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, out_data, nc,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_simple() {
        // Source mesh: 3 points with scalar values.
        let mut source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        source
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Temperature",
                vec![100.0, 200.0, 300.0],
                1,
            )));

        // Target mesh: single point near the first source point.
        let target = PolyData::from_triangles(
            vec![[0.1, 0.1, 0.0], [10.1, 0.1, 0.0], [5.1, 10.1, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = interpolate_nearest(&source, &target, "Temperature");
        let arr = result.point_data().get_array("Temperature").unwrap();
        assert_eq!(arr.num_tuples(), 3);

        let mut val = [0.0f64; 1];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 100.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 200.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut val);
        assert!((val[0] - 300.0).abs() < 1e-10);
    }

    #[test]
    fn interpolate_missing_array() {
        let source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let target = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = interpolate_nearest(&source, &target, "NoSuchArray");
        // Should just return a clone of target with no extra arrays.
        assert_eq!(result.points.len(), 3);
        assert!(result.point_data().get_array("NoSuchArray").is_none());
    }

    #[test]
    fn interpolate_multi_component() {
        let mut source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        source
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Vec2",
                vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                2,
            )));

        let target = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = interpolate_nearest(&source, &target, "Vec2");
        let arr = result.point_data().get_array("Vec2").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut val = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-10);
        assert!((val[1] - 2.0).abs() < 1e-10);
    }
}
