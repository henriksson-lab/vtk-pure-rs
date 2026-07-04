use crate::data::{AnyDataArray, DataArray, DataSetAttributes, KdTree, Points, PolyData};

/// Poisson-disk subsampling of a point set.
///
/// Greedily selects points such that no two selected points are closer
/// than `min_distance`. Produces a well-spaced subset.
pub fn poisson_disk_sample(input: &PolyData, min_distance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut selected: Vec<usize> = Vec::new();
    let mut already_processed = vec![false; n];
    let locator = KdTree::build(&pts);

    // Process in a shuffled candidate order, matching VTK's dart-throwing flow.
    let mut order: Vec<usize> = (0..n).collect();
    let mut rng_state = 1u64;
    for i in (1..n).rev() {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng_state >> 33) as usize % (i + 1);
        order.swap(i, j);
    }

    for &idx in &order {
        if !already_processed[idx] {
            selected.push(idx);
            for (neighbor, _) in locator.find_within_radius(pts[idx], min_distance) {
                already_processed[neighbor] = true;
            }
        }
    }
    selected.sort_unstable();

    let mut out_pts = Points::<f64>::new();
    for &src_idx in &selected {
        out_pts.push(pts[src_idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    copy_point_data(input, &mut pd, &selected);
    pd
}

fn copy_point_data(input: &PolyData, output: &mut PolyData, selected: &[usize]) {
    for array in input.point_data().iter() {
        if array.num_tuples() != input.points.len() {
            continue;
        }
        let Some(subset) = subset_array(array, selected) else {
            continue;
        };
        let name = subset.name().to_string();
        output.point_data_mut().add_array(subset);
        copy_active_attribute(input.point_data(), output.point_data_mut(), &name);
    }
}

fn subset_array(array: &AnyDataArray, selected: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            let nc = a.num_components();
            let mut data = Vec::with_capacity(selected.len() * nc);
            for &idx in selected {
                if idx >= a.num_tuples() {
                    return None;
                }
                data.extend_from_slice(a.tuple(idx));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                a.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(_) => subset_variant!(F32),
        AnyDataArray::F64(_) => subset_variant!(F64),
        AnyDataArray::I8(_) => subset_variant!(I8),
        AnyDataArray::I16(_) => subset_variant!(I16),
        AnyDataArray::I32(_) => subset_variant!(I32),
        AnyDataArray::I64(_) => subset_variant!(I64),
        AnyDataArray::U8(_) => subset_variant!(U8),
        AnyDataArray::U16(_) => subset_variant!(U16),
        AnyDataArray::U32(_) => subset_variant!(U32),
        AnyDataArray::U64(_) => subset_variant!(U64),
    }
}

fn copy_active_attribute(source: &DataSetAttributes, target: &mut DataSetAttributes, name: &str) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_maintained() {
        let mut pd = PolyData::new();
        for i in 0..50 {
            pd.points.push([(i as f64) * 0.1, 0.0, 0.0]);
        }

        let result = poisson_disk_sample(&pd, 0.5);
        assert!(result.points.len() < 50);
        assert!(result.points.len() > 3);

        // Verify min distance
        for i in 0..result.points.len() {
            for j in i + 1..result.points.len() {
                let a = result.points.get(i);
                let b = result.points.get(j);
                let d =
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                assert!(d >= 0.49, "d={} between {} and {}", d, i, j);
            }
        }
    }

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = poisson_disk_sample(&pd, 1.0);
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.verts.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = poisson_disk_sample(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn zero_radius_uses_locator_and_discards_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = poisson_disk_sample(&pd, 0.0);

        assert_eq!(result.points.len(), 2);
    }
}
