//! ResampleWithDataSet — copy scalar data from source to target via nearest-neighbor lookup.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// For each point in `target`, find the nearest point in `source` and copy
/// the named scalar array. Returns a clone of `target` with the interpolated array.
pub fn resample_with_dataset(source: &PolyData, target: &PolyData, array_name: &str) -> PolyData {
    let mut result = target.clone();
    let n_source = source.points.len();
    let n_target = target.points.len();

    if n_source == 0 || n_target == 0 {
        return result;
    }

    let arr = match source.point_data().get_array(array_name) {
        Some(a) => a,
        None => return result,
    };

    let nc = arr.num_components();
    let mut out_data = vec![0.0f64; n_target * nc];

    for ti in 0..n_target {
        let tp = target.points.get(ti);

        // Brute-force nearest-neighbor
        let mut best_dist = f64::MAX;
        let mut best_idx = 0;
        for si in 0..n_source {
            let sp = source.points.get(si);
            let d = (tp[0] - sp[0]).powi(2) + (tp[1] - sp[1]).powi(2) + (tp[2] - sp[2]).powi(2);
            if d < best_dist {
                best_dist = d;
                best_idx = si;
            }
        }

        let mut buf = vec![0.0f64; nc];
        arr.tuple_as_f64(best_idx, &mut buf);
        for c in 0..nc {
            out_data[ti * nc + c] = buf[c];
        }
    }

    let out_arr = AnyDataArray::F64(DataArray::from_vec(array_name, out_data, nc));
    result.point_data_mut().add_array(out_arr);
    if nc == 1 {
        result.point_data_mut().set_active_scalars(array_name);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_neighbor_resample() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([10.0, 0.0, 0.0]);
        let scalars = DataArray::from_vec("temp", vec![100.0, 200.0], 1);
        source
            .point_data_mut()
            .add_array(AnyDataArray::F64(scalars));

        let mut target = PolyData::new();
        target.points.push([1.0, 0.0, 0.0]); // closer to source[0]
        target.points.push([9.0, 0.0, 0.0]); // closer to source[1]

        let result = resample_with_dataset(&source, &target, "temp");
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 100.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 200.0);
    }

    #[test]
    fn missing_array_returns_clone() {
        let source = PolyData::new();
        let target = PolyData::new();
        let result = resample_with_dataset(&source, &target, "nonexistent");
        assert_eq!(result.points.len(), 0);
    }
}
