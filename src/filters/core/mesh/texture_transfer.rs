//! Transfer texture coordinates and point data between meshes using
//! closest-point projection.

use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::types::Scalar;

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
    let active_scalars = pd.scalars().map(|array| array.name().to_string());
    let active_vectors = pd.vectors().map(|array| array.name().to_string());
    let active_normals = pd.normals().map(|array| array.name().to_string());
    let active_tcoords = pd.tcoords().map(|array| array.name().to_string());
    let active_tensors = pd.tensors().map(|array| array.name().to_string());
    let active_global_ids = pd.global_ids().map(|array| array.name().to_string());
    let active_pedigree_ids = pd.pedigree_ids().map(|array| array.name().to_string());
    let active_edge_flags = pd.edge_flags().map(|array| array.name().to_string());
    let active_tangents = pd.tangents().map(|array| array.name().to_string());
    let active_rational_weights = pd.rational_weights().map(|array| array.name().to_string());
    let active_higher_order_degrees = pd
        .higher_order_degrees()
        .map(|array| array.name().to_string());
    let active_process_ids = pd.process_ids().map(|array| array.name().to_string());

    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            if arr.num_tuples() != ns {
                continue;
            }
            result
                .point_data_mut()
                .add_array(copy_array_tuples(arr, &closest));
        }
    }

    let result_pd = result.point_data_mut();
    if let Some(name) = active_scalars {
        result_pd.set_active_scalars(&name);
    }
    if let Some(name) = active_vectors {
        result_pd.set_active_vectors(&name);
    }
    if let Some(name) = active_normals {
        result_pd.set_active_normals(&name);
    }
    if let Some(name) = active_tcoords {
        result_pd.set_active_tcoords(&name);
    }
    if let Some(name) = active_tensors {
        result_pd.set_active_tensors(&name);
    }
    if let Some(name) = active_global_ids {
        result_pd.set_active_global_ids(&name);
    }
    if let Some(name) = active_pedigree_ids {
        result_pd.set_active_pedigree_ids(&name);
    }
    if let Some(name) = active_edge_flags {
        result_pd.set_active_edge_flags(&name);
    }
    if let Some(name) = active_tangents {
        result_pd.set_active_tangents(&name);
    }
    if let Some(name) = active_rational_weights {
        result_pd.set_active_rational_weights(&name);
    }
    if let Some(name) = active_higher_order_degrees {
        result_pd.set_active_higher_order_degrees(&name);
    }
    if let Some(name) = active_process_ids {
        result_pd.set_active_process_ids(&name);
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
    if ns == 0 || nt == 0 || tcoords.num_tuples() != ns {
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
    let name = tcoords.name();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, data, nc)));
    result.point_data_mut().set_active_tcoords(name);
    result
}

fn copy_array_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy_typed {
        ($arr:expr, $variant:path) => {{
            $variant(copy_typed_array($arr, tuple_ids))
        }};
    }

    match array {
        AnyDataArray::F32(a) => copy_typed!(a, AnyDataArray::F32),
        AnyDataArray::F64(a) => copy_typed!(a, AnyDataArray::F64),
        AnyDataArray::I8(a) => copy_typed!(a, AnyDataArray::I8),
        AnyDataArray::I16(a) => copy_typed!(a, AnyDataArray::I16),
        AnyDataArray::I32(a) => copy_typed!(a, AnyDataArray::I32),
        AnyDataArray::I64(a) => copy_typed!(a, AnyDataArray::I64),
        AnyDataArray::U8(a) => copy_typed!(a, AnyDataArray::U8),
        AnyDataArray::U16(a) => copy_typed!(a, AnyDataArray::U16),
        AnyDataArray::U32(a) => copy_typed!(a, AnyDataArray::U32),
        AnyDataArray::U64(a) => copy_typed!(a, AnyDataArray::U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, tuple_ids: &[usize]) -> DataArray<T> {
    let mut output = DataArray::new(array.name(), array.num_components());
    for &tuple_id in tuple_ids {
        output.push_tuple(array.tuple(tuple_id));
    }
    output
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

    #[test]
    fn transfer_preserves_array_scalar_type() {
        let mut source = PolyData::new();
        source.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        source
            .point_data_mut()
            .add_array(AnyDataArray::U16(DataArray::from_vec(
                "labels",
                vec![7u16, 11u16],
                1,
            )));

        let mut target = PolyData::new();
        target.points = Points::from(vec![[0.8, 0.0, 0.0]]);

        let result = transfer_point_data(&source, &target);
        match result.point_data().get_array("labels").unwrap() {
            AnyDataArray::U16(array) => assert_eq!(array.tuple(0), &[11u16]),
            other => panic!("unexpected array type: {:?}", other.scalar_type()),
        }
    }
}
