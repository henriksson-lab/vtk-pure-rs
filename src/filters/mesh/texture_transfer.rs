//! Transfer texture coordinates and point data between meshes using
//! closest-point projection.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Transfer all point data arrays from source to target via closest-point.
pub fn transfer_point_data(source: &PolyData, target: &PolyData) -> PolyData {
    let ns = source.points.len();
    let nt = target.points.len();
    if ns == 0 || nt == 0 {
        return target.clone();
    }

    let src_pts: Vec<[f64; 3]> = (0..ns).map(|i| source.points.get(i)).collect();

    // For each target point, find closest source point
    let closest: Vec<usize> = (0..nt)
        .map(|ti| {
            let tp = target.points.get(ti);
            let mut best = 0;
            let mut best_d = f64::MAX;
            for (si, sp) in src_pts.iter().enumerate() {
                let d = (tp[0] - sp[0]).powi(2) + (tp[1] - sp[1]).powi(2) + (tp[2] - sp[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = si;
                }
            }
            best
        })
        .collect();

    let mut result = target.clone();
    let pd = source.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let name = arr.name().to_string();
            let mut data = Vec::with_capacity(nt * nc);
            let mut buf = vec![0.0f64; nc];
            for &ci in &closest {
                arr.tuple_as_f64(ci, &mut buf);
                data.extend_from_slice(&buf);
            }
            result
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, nc)));
        }
    }
    result
}

/// Transfer texture coordinates from source to target.
pub fn transfer_tcoords(source: &PolyData, target: &PolyData) -> PolyData {
    let tcoords = match source.point_data().tcoords() {
        Some(tc) => tc,
        None => return target.clone(),
    };

    let ns = source.points.len();
    let nt = target.points.len();
    if ns == 0 || nt == 0 {
        return target.clone();
    }

    let src_pts: Vec<[f64; 3]> = (0..ns).map(|i| source.points.get(i)).collect();
    let nc = tcoords.num_components();
    let mut data = Vec::with_capacity(nt * nc);
    let mut buf = vec![0.0f64; nc];

    for ti in 0..nt {
        let tp = target.points.get(ti);
        let mut best = 0;
        let mut best_d = f64::MAX;
        for (si, sp) in src_pts.iter().enumerate() {
            let d = (tp[0] - sp[0]).powi(2) + (tp[1] - sp[1]).powi(2) + (tp[2] - sp[2]).powi(2);
            if d < best_d {
                best_d = d;
                best = si;
            }
        }
        tcoords.tuple_as_f64(best, &mut buf);
        data.extend_from_slice(&buf);
    }

    let mut result = target.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("TCoords", data, nc)));
    result.point_data_mut().set_active_tcoords("TCoords");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    #[test]
    fn transfer_scalars() {
        let mut source = PolyData::new();
        source.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        source
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![100.0, 200.0],
                1,
            )));

        let mut target = PolyData::new();
        target.points = Points::from(vec![[0.1, 0.0, 0.0], [0.9, 0.0, 0.0]]);

        let result = transfer_point_data(&source, &target);
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 100.0); // closest to [0,0,0]
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 200.0); // closest to [1,0,0]
    }

    #[test]
    fn transfer_uv() {
        let mut source = PolyData::new();
        source.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        source
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 1.0, 1.0],
                2,
            )));
        source.point_data_mut().set_active_tcoords("TCoords");

        let mut target = PolyData::new();
        target.points = Points::from(vec![[0.5, 0.0, 0.0]]);

        let result = transfer_tcoords(&source, &target);
        assert!(result.point_data().tcoords().is_some());
    }
}
