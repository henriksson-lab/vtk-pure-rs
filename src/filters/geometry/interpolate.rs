use crate::data::{AnyDataArray, DataArray, PolyData};

/// Interpolate point data from source to target using Inverse Distance Weighting.
///
/// For each target point, computes a weighted average of source values where
/// the weight is `1 / distance^power`. Points within `radius` of the target
/// are considered; if `radius` is 0 or negative, all source points are used.
pub fn interpolate_idw(source: &PolyData, target: &PolyData, power: f64, radius: f64) -> PolyData {
    let mut pd = target.clone();
    let n_source = source.points.len();
    let n_target = target.points.len();

    if n_source == 0 || n_target == 0 {
        return pd;
    }

    let use_radius = radius > 0.0;
    let r2 = radius * radius;

    for arr_idx in 0..source.point_data().num_arrays() {
        let arr = match source.point_data().get_array_by_index(arr_idx) {
            Some(a) => a,
            None => continue,
        };

        let nc = arr.num_components();
        let mut out_data = vec![0.0f64; n_target * nc];

        let mut buf = vec![0.0f64; nc];
        for ti in 0..n_target {
            let tp = target.points.get(ti);
            let mut sum_w = 0.0f64;
            let mut sum_val = vec![0.0f64; nc];

            for si in 0..n_source {
                let sp = source.points.get(si);
                let d2 = (tp[0] - sp[0]) * (tp[0] - sp[0])
                    + (tp[1] - sp[1]) * (tp[1] - sp[1])
                    + (tp[2] - sp[2]) * (tp[2] - sp[2]);

                if use_radius && d2 > r2 {
                    continue;
                }

                let d = d2.sqrt();
                if d < 1e-15 {
                    // Exact match: use this value directly
                    arr.tuple_as_f64(si, &mut buf);
                    for c in 0..nc {
                        out_data[ti * nc + c] = buf[c];
                    }
                    sum_w = -1.0; // sentinel
                    break;
                }

                let w = 1.0 / d.powf(power);
                arr.tuple_as_f64(si, &mut buf);
                for c in 0..nc {
                    sum_val[c] += w * buf[c];
                }
                sum_w += w;
            }

            if sum_w > 0.0 {
                for c in 0..nc {
                    out_data[ti * nc + c] = sum_val[c] / sum_w;
                }
            }
            // If sum_w == -1.0, exact match was already written
            // If sum_w == 0.0, no neighbors found, leave as 0
        }

        let out_arr = AnyDataArray::F64(DataArray::from_vec(arr.name(), out_data, nc));
        pd.point_data_mut().add_array(out_arr);
    }

    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idw_exact_match() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([1.0, 0.0, 0.0]);
        let scalars = DataArray::from_vec("temp", vec![10.0, 20.0], 1);
        source.point_data_mut().add_array(scalars.into());

        let mut target = PolyData::new();
        target.points.push([0.0, 0.0, 0.0]); // exact match with source[0]

        let result = interpolate_idw(&source, &target, 2.0, 0.0);
        let arr = result.point_data().get_array("temp").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn idw_midpoint() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([2.0, 0.0, 0.0]);
        let scalars = DataArray::from_vec("val", vec![0.0, 100.0], 1);
        source.point_data_mut().add_array(scalars.into());

        let mut target = PolyData::new();
        target.points.push([1.0, 0.0, 0.0]); // equidistant

        let result = interpolate_idw(&source, &target, 2.0, 0.0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 50.0).abs() < 1e-10); // equal weights -> average
    }

    #[test]
    fn idw_with_radius() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([10.0, 0.0, 0.0]); // far away
        let scalars = DataArray::from_vec("val", vec![1.0, 999.0], 1);
        source.point_data_mut().add_array(scalars.into());

        let mut target = PolyData::new();
        target.points.push([0.1, 0.0, 0.0]);

        let result = interpolate_idw(&source, &target, 2.0, 2.0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        // Only source[0] is within radius 2, so value should be ~1.0
        assert!((val[0] - 1.0).abs() < 0.5);
    }
}
