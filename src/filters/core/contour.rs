use std::collections::HashMap;

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Extract contour lines at a given isovalue from a PolyData with scalar data.
///
/// For each polygon (typically triangles), finds edges where the scalar field
/// crosses the isovalue and produces line segments connecting those crossing
/// points. This is the 2D analogue of marching cubes.
pub fn contour(input: &PolyData, scalars: &[f64], isovalue: f64) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut out_scalars = DataArray::<f64>::new("contour_scalars", 1);

    if scalars.len() < input.points.len() {
        return PolyData::new();
    }

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 3 {
            continue;
        }

        // Collect crossing points on edges of this polygon.
        let mut crossing_pts: Vec<usize> = Vec::new();
        let mut exact_vertex_pts: HashMap<usize, usize> = HashMap::new();

        for i in 0..n {
            let id0 = cell[i] as usize;
            let id1 = cell[(i + 1) % n] as usize;
            let s0 = scalars[id0];
            let s1 = scalars[id1];
            let d0 = s0 - isovalue;
            let d1 = s1 - isovalue;

            if d0 == 0.0 {
                let idx = *exact_vertex_pts.entry(id0).or_insert_with(|| {
                    let idx = out_points.len();
                    out_points.push(input.points.get(id0));
                    out_scalars.push_tuple(&[isovalue]);
                    idx
                });
                if !crossing_pts.contains(&idx) {
                    crossing_pts.push(idx);
                }
            }
            if d1 == 0.0 {
                let idx = *exact_vertex_pts.entry(id1).or_insert_with(|| {
                    let idx = out_points.len();
                    out_points.push(input.points.get(id1));
                    out_scalars.push_tuple(&[isovalue]);
                    idx
                });
                if !crossing_pts.contains(&idx) {
                    crossing_pts.push(idx);
                }
            }

            // Check if isovalue strictly crosses this edge.
            if d0 * d1 < 0.0 {
                let t = (isovalue - s0) / (s1 - s0);
                let p0 = input.points.get(id0);
                let p1 = input.points.get(id1);
                let idx = out_points.len();
                out_points.push([
                    p0[0] + t * (p1[0] - p0[0]),
                    p0[1] + t * (p1[1] - p0[1]),
                    p0[2] + t * (p1[2] - p0[2]),
                ]);
                out_scalars.push_tuple(&[isovalue]);
                crossing_pts.push(idx);
            }
        }

        // For a triangle, we expect exactly 2 crossings (forming one line segment)
        // For general polygons, connect crossings in pairs
        if crossing_pts.len() >= 2 {
            for pair in crossing_pts.chunks(2) {
                if pair.len() == 2 {
                    out_lines.push_cell(&[pair[0] as i64, pair[1] as i64]);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.point_data_mut().add_array(out_scalars.into());
    pd
}

/// Extract multiple contour lines at evenly spaced isovalues.
pub fn contour_range(
    input: &PolyData,
    scalars: &[f64],
    min_value: f64,
    max_value: f64,
    num_contours: usize,
) -> PolyData {
    if num_contours == 0 {
        return PolyData::new();
    }

    let mut results: Vec<PolyData> = Vec::new();
    for i in 0..num_contours {
        let t = if num_contours == 1 {
            0.5
        } else {
            i as f64 / (num_contours - 1) as f64
        };
        let isovalue = min_value + t * (max_value - min_value);
        results.push(contour(input, scalars, isovalue));
    }

    // Merge all results
    if results.len() == 1 {
        return results.into_iter().next().unwrap();
    }

    let refs: Vec<&PolyData> = results.iter().collect();
    crate::filters::core::append::append(&refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contour_on_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = vec![0.0, 1.0, 0.5];
        let result = contour(&pd, &scalars, 0.25);
        // Isovalue 0.25 crosses edge 0-1 (at t=0.25) and edge 0-2 (at t=0.5)
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn contour_no_crossing() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = vec![1.0, 2.0, 3.0];
        let result = contour(&pd, &scalars, 5.0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn contour_range_multiple() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = vec![0.0, 1.0, 0.5];
        let result = contour_range(&pd, &scalars, 0.1, 0.9, 3);
        // 3 contour values: 0.1, 0.5, 0.9
        assert!(result.lines.num_cells() >= 3);
    }

    #[test]
    fn contour_through_exact_vertex() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = vec![0.0, 1.0, 0.5];
        let result = contour(&pd, &scalars, 0.5);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }
}
