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

    let mut out_points = input.points.clone();
    let mut out_point_scalars = scalar_data.clone();
    let mut out_verts = CellArray::new();
    let mut out_lines = CellArray::new();
    let mut out_polys = CellArray::new();
    let mut band_ids: Vec<f64> = Vec::new();
    let mut vtk_scalars: Vec<f32> = Vec::new();

    for cell in input.verts.iter() {
        for &pid in cell {
            let idx = compute_clipped_index(scalar_data[pid as usize], &clip_values);
            out_verts.push_cell(&[pid]);
            band_ids.push(idx as f64);
            vtk_scalars.push(idx as f32);
        }
    }

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() - 1 {
            insert_banded_line_segment(
                cell[i],
                cell[i + 1],
                input,
                &scalar_data,
                &clip_values,
                &mut out_points,
                &mut out_point_scalars,
                &mut out_lines,
                &mut band_ids,
                &mut vtk_scalars,
            );
        }
    }

    let work_polys = polys_with_decomposed_strips(input);

    for cell in work_polys.iter() {
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

                let clipped = clip_triangle_to_band(
                    [cell[0], cell[i], cell[i + 1]],
                    [v0, v1, v2],
                    [s0, s1, s2],
                    lo,
                    hi,
                    &mut out_points,
                    &mut out_point_scalars,
                );
                if clipped.len() >= 3 {
                    out_polys.push_cell(&clipped);
                    band_ids.push(bi as f64);
                    vtk_scalars.push(bi as f32);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.lines = out_lines;
    pd.polys = out_polys;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            out_point_scalars,
            1,
        )));
    pd.point_data_mut().set_active_scalars(scalars);
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "BandIndex",
            band_ids,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F32(DataArray::from_vec(
            "Scalars",
            vtk_scalars,
            1,
        )));
    pd.cell_data_mut().set_active_scalars("Scalars");
    pd
}

/// Clip a triangle to the scalar band [lo, hi].
/// Returns the polygon vertices that lie within the band.
fn clip_triangle_to_band(
    ids: [i64; 3],
    verts: [[f64; 3]; 3],
    scalars: [f64; 3],
    lo: f64,
    hi: f64,
    out_points: &mut Points<f64>,
    out_point_scalars: &mut Vec<f64>,
) -> Vec<i64> {
    // Start with the triangle, then clip by lo (keep >= lo), then by hi (keep <= hi)
    let mut polygon = vec![
        BandPoint {
            id: ids[0],
            point: verts[0],
            scalar: scalars[0],
        },
        BandPoint {
            id: ids[1],
            point: verts[1],
            scalar: scalars[1],
        },
        BandPoint {
            id: ids[2],
            point: verts[2],
            scalar: scalars[2],
        },
    ];

    polygon = clip_polygon_by_threshold(&polygon, lo, true, out_points, out_point_scalars);
    if polygon.len() < 3 {
        return vec![];
    }
    polygon = clip_polygon_by_threshold(&polygon, hi, false, out_points, out_point_scalars);
    polygon.into_iter().map(|p| p.id).collect()
}

/// Sutherland-Hodgman clip of a polygon by a scalar threshold.
/// If `keep_above` is true, keeps vertices where scalar >= threshold.
/// If false, keeps vertices where scalar <= threshold.
fn clip_polygon_by_threshold(
    polygon: &[BandPoint],
    threshold: f64,
    keep_above: bool,
    out_points: &mut Points<f64>,
    out_point_scalars: &mut Vec<f64>,
) -> Vec<BandPoint> {
    let n = polygon.len();
    if n == 0 {
        return vec![];
    }

    let inside = |s: f64| -> bool {
        if keep_above {
            s >= threshold
        } else {
            s <= threshold
        }
    };

    let mut out = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        let pi = polygon[i];
        let pj = polygon[j];
        let si = pi.scalar;
        let sj = pj.scalar;

        let i_in = inside(si);
        let j_in = inside(sj);

        if i_in {
            out.push(pi);
        }

        if i_in != j_in {
            // Edge crosses the threshold
            let ds = sj - si;
            if ds.abs() > 1e-15 {
                let t = (threshold - si) / ds;
                let t = t.clamp(0.0, 1.0);
                let point = lerp3(pi.point, pj.point, t);
                let id = out_points.len() as i64;
                out_points.push(point);
                out_point_scalars.push(threshold);
                out.push(BandPoint {
                    id,
                    point,
                    scalar: threshold,
                });
            }
        }
    }

    out
}

#[derive(Clone, Copy)]
struct BandPoint {
    id: i64,
    point: [f64; 3],
    scalar: f64,
}

fn insert_banded_line_segment(
    p1: i64,
    p2: i64,
    input: &PolyData,
    scalar_data: &[f64],
    clip_values: &[f64],
    out_points: &mut Points<f64>,
    out_point_scalars: &mut Vec<f64>,
    out_lines: &mut CellArray,
    band_ids: &mut Vec<f64>,
    vtk_scalars: &mut Vec<f32>,
) {
    let s1 = scalar_data[p1 as usize];
    let s2 = scalar_data[p2 as usize];
    let mut pts = vec![(p1, s1)];
    let low = s1.min(s2);
    let high = s1.max(s2);

    for &value in clip_values {
        if value <= low || value >= high {
            continue;
        }
        let t = (value - s1) / (s2 - s1);
        let new_id = out_points.len() as i64;
        out_points.push(lerp3(
            input.points.get(p1 as usize),
            input.points.get(p2 as usize),
            t,
        ));
        out_point_scalars.push(value);
        pts.push((new_id, value));
    }
    pts.push((p2, s2));
    pts.sort_by(|a, b| {
        let ta = if (s2 - s1).abs() > 1e-15 {
            (a.1 - s1) / (s2 - s1)
        } else {
            0.0
        };
        let tb = if (s2 - s1).abs() > 1e-15 {
            (b.1 - s1) / (s2 - s1)
        } else {
            0.0
        };
        ta.partial_cmp(&tb).unwrap()
    });

    for pair in pts.windows(2) {
        let value = pair[0].1.min(pair[1].1);
        let idx = compute_clipped_index(value, clip_values);
        out_lines.push_cell(&[pair[0].0, pair[1].0]);
        band_ids.push(idx as f64);
        vtk_scalars.push(idx as f32);
    }
}

fn compute_clipped_index(scalar: f64, clip_values: &[f64]) -> usize {
    match clip_values.binary_search_by(|v| v.partial_cmp(&scalar).unwrap()) {
        Ok(i) => i.min(clip_values.len().saturating_sub(2)),
        Err(i) => i.saturating_sub(1).min(clip_values.len().saturating_sub(2)),
    }
}

fn polys_with_decomposed_strips(input: &PolyData) -> CellArray {
    if input.strips.is_empty() {
        return input.polys.clone();
    }

    let mut polys = input.polys.clone();
    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            if i % 2 == 0 {
                polys.push_cell(&[strip[i], strip[i + 1], strip[i + 2]]);
            } else {
                polys.push_cell(&[strip[i + 1], strip[i], strip[i + 2]]);
            }
        }
    }
    polys
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
