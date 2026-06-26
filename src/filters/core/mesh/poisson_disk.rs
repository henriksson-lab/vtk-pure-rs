use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

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
    let d2 = min_distance * min_distance;

    let mut selected: Vec<usize> = Vec::new();
    // Process in random-ish order (use point index shuffled by hash)
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| {
        let p = pts[i];
        ((p[0] * 73856093.0) as i64 ^ (p[1] * 19349663.0) as i64 ^ (p[2] * 83492791.0) as i64)
            .wrapping_abs()
    });

    for &idx in &order {
        let p = pts[idx];

        // Check against all selected points
        let too_close = selected.iter().any(|&s| {
            let q = pts[s];
            (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) < d2
        });

        if !too_close {
            selected.push(idx);
        }
    }
    selected.sort_unstable();

    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    for &src_idx in &selected {
        let idx = out_pts.len() as i64;
        out_pts.push(pts[src_idx]);
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
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
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = poisson_disk_sample(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
