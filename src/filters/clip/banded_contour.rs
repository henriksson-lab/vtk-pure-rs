use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate filled contour bands from scalar-colored triangle mesh.
///
/// Given a PolyData with a scalar array and a set of contour values,
/// this filter produces filled polygonal bands between consecutive
/// contour levels. Each band is a region of the surface where the
/// scalar value falls between two consecutive contour values.
///
/// The output has a "BandIndex" cell data array indicating which band
/// each cell belongs to (0-indexed).
pub fn banded_contour(input: &PolyData, scalars: &str, values: &[f64]) -> PolyData {
    if values.is_empty() {
        return PolyData::new();
    }

    let scalar_arr = match input.point_data().get_array(scalars) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let n_pts = input.points.len();
    let mut scalar_data = vec![0.0f64; n_pts];
    let mut buf = [0.0f64];
    for (i, val) in scalar_data.iter_mut().enumerate() {
        scalar_arr.tuple_as_f64(i, &mut buf);
        *val = buf[0];
    }

    let scalar_range = scalar_data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |range, &s| {
            (range.0.min(s), range.1.max(s))
        });
    if !scalar_range.0.is_finite() || !scalar_range.1.is_finite() {
        return PolyData::new();
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut clip_values = Vec::with_capacity(sorted.len() + 2);
    if scalar_range.0 < sorted[0] {
        clip_values.push(scalar_range.0);
    }
    clip_values.extend(sorted);
    if scalar_range.1 > *clip_values.last().unwrap() {
        clip_values.push(scalar_range.1);
    }
    clip_values.dedup_by(|a, b| (*a - *b).abs() <= f64::EPSILON);
    if clip_values.len() < 2 {
        return PolyData::new();
    }

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut band_ids: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Fan-triangulate the cell
        let p0_idx = cell[0] as usize;
        for i in 1..cell.len() - 1 {
            let p1_idx = cell[i] as usize;
            let p2_idx = cell[i + 1] as usize;

            let s0 = scalar_data[p0_idx];
            let s1 = scalar_data[p1_idx];
            let s2 = scalar_data[p2_idx];

            let v0 = input.points.get(p0_idx);
            let v1 = input.points.get(p1_idx);
            let v2 = input.points.get(p2_idx);

            // For each band, clip the triangle to that band
            for bi in 0..clip_values.len() - 1 {
                let lo = clip_values[bi];
                let hi = clip_values[bi + 1];

                let clipped = clip_triangle_to_band(v0, v1, v2, s0, s1, s2, lo, hi);
                if clipped.len() >= 3 {
                    let base = out_points.len() as i64;
                    for p in &clipped {
                        out_points.push(*p);
                    }
                    // Fan-triangulate the clipped polygon
                    for j in 1..clipped.len() - 1 {
                        out_polys.push_cell(&[base, base + j as i64, base + (j + 1) as i64]);
                        band_ids.push(bi as f64);
                    }
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BandIndex",
            band_ids,
            1,
        )));
    pd
}

/// Clip a triangle to the scalar band [lo, hi].
/// Returns the polygon vertices that lie within the band.
fn clip_triangle_to_band(
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
    s0: f64,
    s1: f64,
    s2: f64,
    lo: f64,
    hi: f64,
) -> Vec<[f64; 3]> {
    // Start with the triangle, then clip by lo (keep >= lo), then by hi (keep <= hi)
    let verts = vec![v0, v1, v2];
    let scalars = vec![s0, s1, s2];

    let (verts, scalars) = clip_polygon_by_threshold(&verts, &scalars, lo, true);
    if verts.len() < 3 {
        return vec![];
    }
    let (verts, _) = clip_polygon_by_threshold(&verts, &scalars, hi, false);
    verts
}

/// Sutherland-Hodgman clip of a polygon by a scalar threshold.
/// If `keep_above` is true, keeps vertices where scalar >= threshold.
/// If false, keeps vertices where scalar <= threshold.
fn clip_polygon_by_threshold(
    verts: &[[f64; 3]],
    scalars: &[f64],
    threshold: f64,
    keep_above: bool,
) -> (Vec<[f64; 3]>, Vec<f64>) {
    let n = verts.len();
    if n == 0 {
        return (vec![], vec![]);
    }

    let inside = |s: f64| -> bool {
        if keep_above {
            s >= threshold
        } else {
            s <= threshold
        }
    };

    let mut out_verts = Vec::new();
    let mut out_scalars = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        let si = scalars[i];
        let sj = scalars[j];
        let vi = verts[i];
        let vj = verts[j];

        let i_in = inside(si);
        let j_in = inside(sj);

        if i_in {
            out_verts.push(vi);
            out_scalars.push(si);
        }

        if i_in != j_in {
            // Edge crosses the threshold
            let ds = sj - si;
            if ds.abs() > 1e-15 {
                let t = (threshold - si) / ds;
                let t = t.clamp(0.0, 1.0);
                out_verts.push(lerp3(vi, vj, t));
                out_scalars.push(threshold);
            }
        }
    }

    (out_verts, out_scalars)
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tri_with_scalars() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "scalars",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        pd.point_data_mut().set_active_scalars("scalars");
        pd
    }

    #[test]
    fn single_band_covers_all() {
        let pd = make_tri_with_scalars();
        let result = banded_contour(&pd, "scalars", &[0.0, 1.0]);
        assert!(result.polys.num_cells() >= 1);
    }

    #[test]
    fn two_bands() {
        let pd = make_tri_with_scalars();
        let result = banded_contour(&pd, "scalars", &[0.0, 0.5, 1.0]);
        assert!(result.polys.num_cells() >= 2);
    }

    #[test]
    fn no_scalars() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        let result = banded_contour(&pd, "missing", &[0.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn single_value_adds_range_bands() {
        let pd = make_tri_with_scalars();
        let result = banded_contour(&pd, "scalars", &[0.5]);
        assert!(result.polys.num_cells() >= 2);
    }

    #[test]
    fn band_index_array() {
        let pd = make_tri_with_scalars();
        let result = banded_contour(&pd, "scalars", &[0.0, 0.5, 1.0]);
        let band = result.cell_data().get_array("BandIndex");
        assert!(band.is_some());
    }
}
