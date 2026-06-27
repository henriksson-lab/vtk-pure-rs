use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};

/// Interpolate source data at probe point locations.
///
/// For each point in `probe`, finds the nearest point in `source` and
/// copies its scalar data. Uses a brute-force nearest-neighbor search.
pub fn probe(source: &PolyData, probe_points: &PolyData) -> PolyData {
    let mut pd = probe_points.clone();
    let n_source = source.points.len();
    let n_probe = probe_points.points.len();

    if n_source == 0 || n_probe == 0 {
        return pd;
    }

    // Pre-compute nearest source index using flat slice access — matches VTK C++ speed.
    // Separating nearest-search from data copy avoids redundant work with multiple arrays.
    let src_pts = source.points.as_flat_slice();
    let prb_pts = probe_points.points.as_flat_slice();
    let mut nearest = Vec::with_capacity(n_probe);

    for pi in 0..n_probe {
        let pb = pi * 3;
        let px = prb_pts[pb];
        let py = prb_pts[pb + 1];
        let pz = prb_pts[pb + 2];

        let mut best_dist = f64::MAX;
        let mut best_idx = 0usize;
        for si in 0..n_source {
            let sb = si * 3;
            let dx = px - src_pts[sb];
            let dy = py - src_pts[sb + 1];
            let dz = pz - src_pts[sb + 2];
            let d = dx * dx + dy * dy + dz * dz;
            if d < best_dist {
                best_dist = d;
                best_idx = si;
            }
        }
        nearest.push(best_idx);
    }

    copy_point_data_by_ids(source.point_data(), pd.point_data_mut(), &nearest);

    pd
}

fn copy_point_data_by_ids(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if let Some(subset) = subset_array(array, tuple_ids) {
            let name = subset.name().to_string();
            target.add_array(subset);
            copy_active_attribute_for_array(source, target, &name);
        }
    }
}

fn copy_active_attribute_for_array(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    name: &str,
) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.normals().map(|a| a.name()) == Some(name) {
        target.set_active_normals(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
    if source.tensors().map(|a| a.name()) == Some(name) {
        target.set_active_tensors(name);
    }
    if source.global_ids().map(|a| a.name()) == Some(name) {
        target.set_active_global_ids(name);
    }
    if source.pedigree_ids().map(|a| a.name()) == Some(name) {
        target.set_active_pedigree_ids(name);
    }
    if source.edge_flags().map(|a| a.name()) == Some(name) {
        target.set_active_edge_flags(name);
    }
    if source.tangents().map(|a| a.name()) == Some(name) {
        target.set_active_tangents(name);
    }
    if source.rational_weights().map(|a| a.name()) == Some(name) {
        target.set_active_rational_weights(name);
    }
    if source.higher_order_degrees().map(|a| a.name()) == Some(name) {
        target.set_active_higher_order_degrees(name);
    }
    if source.process_ids().map(|a| a.name()) == Some(name) {
        target.set_active_process_ids(name);
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident, $a:expr) => {{
            let nc = $a.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                let from = tuple_id.checked_mul(nc)?;
                let to = from.checked_add(nc)?;
                data.extend_from_slice($a.as_slice().get(from..to)?);
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $a.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(a) => subset_variant!(F32, a),
        AnyDataArray::F64(a) => subset_variant!(F64, a),
        AnyDataArray::I8(a) => subset_variant!(I8, a),
        AnyDataArray::I16(a) => subset_variant!(I16, a),
        AnyDataArray::I32(a) => subset_variant!(I32, a),
        AnyDataArray::I64(a) => subset_variant!(I64, a),
        AnyDataArray::U8(a) => subset_variant!(U8, a),
        AnyDataArray::U16(a) => subset_variant!(U16, a),
        AnyDataArray::U32(a) => subset_variant!(U32, a),
        AnyDataArray::U64(a) => subset_variant!(U64, a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_nearest_neighbor() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([1.0, 0.0, 0.0]);
        source.points.push([2.0, 0.0, 0.0]);
        let scalars = DataArray::from_vec("temp", vec![10.0, 20.0, 30.0], 1);
        source.point_data_mut().add_array(scalars.into());

        let mut probe_pts = PolyData::new();
        probe_pts.points.push([0.1, 0.0, 0.0]); // nearest to point 0
        probe_pts.points.push([1.6, 0.0, 0.0]); // nearest to point 2

        let result = probe(&source, &probe_pts);
        let arr = result.point_data().get_array("temp").unwrap();

        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 10.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 30.0).abs() < 1e-10);
    }

    #[test]
    fn probe_multicomponent() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([1.0, 0.0, 0.0]);
        let vecs = DataArray::from_vec("vel", vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0], 3);
        source.point_data_mut().add_array(vecs.into());

        let mut probe_pts = PolyData::new();
        probe_pts.points.push([0.9, 0.0, 0.0]); // nearest to point 1

        let result = probe(&source, &probe_pts);
        let arr = result.point_data().get_array("vel").unwrap();
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn probe_preserves_array_type_and_active_scalars() {
        let mut source = PolyData::new();
        source.points.push([0.0, 0.0, 0.0]);
        source.points.push([1.0, 0.0, 0.0]);
        source
            .point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "labels",
                vec![7, 9],
                1,
            )));
        source.point_data_mut().set_active_scalars("labels");

        let mut probe_pts = PolyData::new();
        probe_pts.points.push([0.8, 0.0, 0.0]);

        let result = probe(&source, &probe_pts);
        assert!(matches!(
            result.point_data().scalars().unwrap(),
            AnyDataArray::U8(_)
        ));
        let mut value = [0.0f64; 1];
        result
            .point_data()
            .scalars()
            .unwrap()
            .tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 9.0);
    }
}
