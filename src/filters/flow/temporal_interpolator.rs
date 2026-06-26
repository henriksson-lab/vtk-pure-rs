use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Linearly interpolate between two PolyData at parameter t in [0, 1].
///
/// Both inputs must have the same number of points and the same topology.
/// Point positions and all matching scalar arrays are interpolated.
/// t=0 returns `a`, t=1 returns `b`.
pub fn temporal_interpolator(a: &PolyData, b: &PolyData, t: f64) -> PolyData {
    let t = t.clamp(0.0, 1.0);
    let n = a.points.len();

    if n != b.points.len() {
        return a.clone();
    }

    // Interpolate points
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let pa = a.points.get(i);
        let pb = b.points.get(i);
        points.push([
            pa[0] + t * (pb[0] - pa[0]),
            pa[1] + t * (pb[1] - pa[1]),
            pa[2] + t * (pb[2] - pa[2]),
        ]);
    }

    let mut pd = a.clone();
    pd.points = points;

    // Interpolate matching point arrays, preserving active attribute designations.
    let mut new_attrs = a.point_data().clone();
    for i in 0..a.point_data().num_arrays() {
        let arr_a = a.point_data().get_array_by_index(i).unwrap();
        if let Some(arr_b) = b.point_data().get_array(arr_a.name()) {
            if arr_a.num_tuples() == arr_b.num_tuples()
                && arr_a.num_components() == arr_b.num_components()
            {
                let nc = arr_a.num_components();
                let nt = arr_a.num_tuples();
                let mut interp = Vec::with_capacity(nt * nc);
                let mut ba = vec![0.0f64; nc];
                let mut bb = vec![0.0f64; nc];
                for ti in 0..nt {
                    arr_a.tuple_as_f64(ti, &mut ba);
                    arr_b.tuple_as_f64(ti, &mut bb);
                    for c in 0..nc {
                        interp.push(ba[c] + t * (bb[c] - ba[c]));
                    }
                }
                new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                    arr_a.name(),
                    interp,
                    nc,
                )));
            }
        }
    }
    *pd.point_data_mut() = new_attrs;

    // vtkTemporalInterpolator applies the same name/component/tuple matching to cell data.
    let mut new_cell_attrs = a.cell_data().clone();
    for i in 0..a.cell_data().num_arrays() {
        let arr_a = a.cell_data().get_array_by_index(i).unwrap();
        if let Some(arr_b) = b.cell_data().get_array(arr_a.name()) {
            if arr_a.num_tuples() == arr_b.num_tuples()
                && arr_a.num_components() == arr_b.num_components()
            {
                let nc = arr_a.num_components();
                let nt = arr_a.num_tuples();
                let mut interp = Vec::with_capacity(nt * nc);
                let mut ba = vec![0.0f64; nc];
                let mut bb = vec![0.0f64; nc];
                for ti in 0..nt {
                    arr_a.tuple_as_f64(ti, &mut ba);
                    arr_b.tuple_as_f64(ti, &mut bb);
                    for c in 0..nc {
                        interp.push(ba[c] + t * (bb[c] - ba[c]));
                    }
                }
                new_cell_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                    arr_a.name(),
                    interp,
                    nc,
                )));
            }
        }
    }
    *pd.cell_data_mut() = new_cell_attrs;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midpoint_interpolation() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([0.0, 0.0, 2.0]);
        b.points.push([1.0, 0.0, 2.0]);

        let result = temporal_interpolator(&a, &b, 0.5);
        let p = result.points.get(0);
        assert!((p[2] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn t_zero_is_a() {
        let mut a = PolyData::new();
        a.points.push([1.0, 2.0, 3.0]);
        let mut b = PolyData::new();
        b.points.push([10.0, 20.0, 30.0]);

        let result = temporal_interpolator(&a, &b, 0.0);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn t_one_is_b() {
        let mut a = PolyData::new();
        a.points.push([1.0, 2.0, 3.0]);
        let mut b = PolyData::new();
        b.points.push([10.0, 20.0, 30.0]);

        let result = temporal_interpolator(&a, &b, 1.0);
        let p = result.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn interpolate_scalars() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![100.0],
                1,
            )));

        let mut b = PolyData::new();
        b.points.push([0.0, 0.0, 0.0]);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![200.0],
                1,
            )));

        let result = temporal_interpolator(&a, &b, 0.25);
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 125.0).abs() < 1e-10);
    }

    #[test]
    fn mismatched_sizes() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);

        let result = temporal_interpolator(&a, &b, 0.5);
        // Falls back to a
        assert_eq!(result.points.len(), 1);
    }
}
