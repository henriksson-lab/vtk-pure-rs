use crate::data::{CellArray, Points, PolyData};

/// Extract the volume (surface) between two isovalues.
///
/// Clips the input mesh to keep only the region where scalar values
/// are between `lower` and `upper`. Equivalent to applying two threshold
/// clips. Triangles crossing the isovalue boundaries are clipped.
pub fn iso_volume(input: &PolyData, scalars: &str, lower: f64, upper: f64) -> PolyData {
    let scalar_arr = match input.point_data().get_array(scalars) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    let mut svals = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for (i, v) in svals.iter_mut().enumerate() {
        scalar_arr.tuple_as_f64(i, &mut buf);
        *v = buf[0];
    }

    let mut out_points = input.points.clone();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Fan-triangulate
        for ti in 1..cell.len() - 1 {
            let ids = [cell[0], cell[ti], cell[ti + 1]];
            let sv = [
                svals[ids[0] as usize],
                svals[ids[1] as usize],
                svals[ids[2] as usize],
            ];
            let verts: Vec<[f64; 3]> = ids
                .iter()
                .map(|&id| input.points.get(id as usize))
                .collect();

            // Clip to keep lower <= s <= upper using Sutherland-Hodgman
            let (v1, s1) = clip_poly_threshold(&verts, &sv.to_vec(), lower, true);
            if v1.len() < 3 {
                continue;
            }
            let (v2, _) = clip_poly_threshold(&v1, &s1, upper, false);
            if v2.len() < 3 {
                continue;
            }

            let base = out_points.len() as i64;
            for p in &v2 {
                out_points.push(*p);
            }
            for i in 1..v2.len() - 1 {
                out_polys.push_cell(&[base, base + i as i64, base + (i + 1) as i64]);
            }
        }
    }

    compact_poly_data(out_points, out_polys)
}

/// Sutherland-Hodgman clip: keep vertices where scalar >= threshold (keep_above)
/// or <= threshold (!keep_above). Returns clipped positions and interpolated scalars.
fn clip_poly_threshold(
    verts: &[[f64; 3]],
    scalars: &[f64],
    threshold: f64,
    keep_above: bool,
) -> (Vec<[f64; 3]>, Vec<f64>) {
    let n = verts.len();
    let mut out_v = Vec::new();
    let mut out_s = Vec::new();

    let inside = |s: f64| -> bool {
        if keep_above {
            s >= threshold
        } else {
            s <= threshold
        }
    };

    for i in 0..n {
        let j = (i + 1) % n;
        let si = scalars[i];
        let sj = scalars[j];

        if inside(si) {
            out_v.push(verts[i]);
            out_s.push(si);
        }

        if inside(si) != inside(sj) {
            let ds = sj - si;
            if ds.abs() > 1e-15 {
                let t = ((threshold - si) / ds).clamp(0.0, 1.0);
                out_v.push(lerp3(verts[i], verts[j], t));
                out_s.push(threshold);
            }
        }
    }

    (out_v, out_s)
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

fn compact_poly_data(points: Points<f64>, polys: CellArray) -> PolyData {
    let mut used = vec![false; points.len()];
    for cell in polys.iter() {
        for &id in cell {
            used[id as usize] = true;
        }
    }

    let mut point_map = vec![0i64; points.len()];
    let mut compact_points = Points::new();
    for (old_id, is_used) in used.into_iter().enumerate() {
        if is_used {
            point_map[old_id] = compact_points.len() as i64;
            compact_points.push(points.get(old_id));
        }
    }

    let mut compact_polys = CellArray::new();
    for cell in polys.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        compact_polys.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = compact_points;
    pd.polys = compact_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn extract_middle_band() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 1.0, 0.0],
                1,
            )));

        let result = iso_volume(&pd, "s", 0.25, 0.75);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn all_inside() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.5, 0.5, 0.5],
                1,
            )));

        let result = iso_volume(&pd, "s", 0.0, 1.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn all_outside() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![5.0, 5.0, 5.0],
                1,
            )));

        let result = iso_volume(&pd, "s", 0.0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn missing_scalars() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = iso_volume(&pd, "missing", 0.0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
