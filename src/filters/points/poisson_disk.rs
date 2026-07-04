//! PoissonDiskSampler — Poisson disk subsampling from existing points.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, Points, PolyData};

/// Poisson disk subsampling using dart-throwing.
///
/// Iterates through points in random order (seeded by `seed`), accepting
/// each point only if no previously accepted point lies within `radius`.
/// Returns a PolyData with the accepted subset and no cells.
pub fn poisson_disk_sample(input: &PolyData, radius: f64, seed: u64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let r2 = radius * radius;

    // Simple LCG PRNG for shuffling
    let mut indices: Vec<usize> = (0..n).collect();
    let mut rng_state = seed.wrapping_add(1);
    // Fisher-Yates shuffle
    for i in (1..n).rev() {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng_state >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }

    let mut accepted: Vec<usize> = Vec::new();

    for &idx in &indices {
        let p = pts[idx];
        let mut too_close = false;
        for &aidx in &accepted {
            let q = pts[aidx];
            let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            if d2 <= r2 {
                too_close = true;
                break;
            }
        }
        if !too_close {
            accepted.push(idx);
        }
    }
    accepted.sort_unstable();

    let mut new_pts = Points::<f64>::new();
    for &idx in &accepted {
        new_pts.push(pts[idx]);
    }

    let mut result = PolyData::new();
    result.points = new_pts;
    copy_point_data(input, &mut result, &accepted);
    result
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
    fn subsamples_points() {
        let mut pd = PolyData::new();
        // Dense grid
        for i in 0..10 {
            for j in 0..10 {
                pd.points.push([i as f64 * 0.1, j as f64 * 0.1, 0.0]);
            }
        }

        let result = poisson_disk_sample(&pd, 0.25, 42);
        // Should have fewer points than input
        assert!(result.points.len() < 100);
        assert!(result.points.len() > 0);

        // Verify minimum distance constraint
        let n = result.points.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = result.points.get(i);
                let b = result.points.get(j);
                let d2 = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
                assert!(
                    d2 >= 0.25 * 0.25 - 1e-10,
                    "points too close: d={}",
                    d2.sqrt()
                );
            }
        }
    }

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = poisson_disk_sample(&pd, 1.0, 0);
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.verts.num_cells(), 0);
    }

    #[test]
    fn copies_point_data_in_output_order() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([20.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "id",
                vec![10, 20, 30],
                1,
            )));

        let result = poisson_disk_sample(&pd, 1.0, 42);
        let ids = result.point_data().get_array("id").unwrap();
        let mut buf = [0.0];
        for i in 0..3 {
            ids.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], (i as f64 + 1.0) * 10.0);
        }
    }
}
