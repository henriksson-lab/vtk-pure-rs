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
    let mut exact_vertex_pts: HashMap<usize, usize> = HashMap::new();
    let mut edge_intersection_pts: HashMap<(usize, usize), usize> = HashMap::new();

    if scalars.len() < input.points.len() {
        return PolyData::new();
    }

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 3 {
            continue;
        }

        if n == 3 {
            contour_triangle(
                [cell[0] as usize, cell[1] as usize, cell[2] as usize],
                input,
                scalars,
                isovalue,
                &mut out_points,
                &mut out_lines,
                &mut out_scalars,
                &mut exact_vertex_pts,
                &mut edge_intersection_pts,
            );
        } else if n == 4 {
            contour_quad(
                [
                    cell[0] as usize,
                    cell[1] as usize,
                    cell[2] as usize,
                    cell[3] as usize,
                ],
                input,
                scalars,
                isovalue,
                &mut out_points,
                &mut out_lines,
                &mut out_scalars,
                &mut exact_vertex_pts,
                &mut edge_intersection_pts,
            );
        } else {
            for i in 1..n - 1 {
                contour_triangle(
                    [cell[0] as usize, cell[i] as usize, cell[i + 1] as usize],
                    input,
                    scalars,
                    isovalue,
                    &mut out_points,
                    &mut out_lines,
                    &mut out_scalars,
                    &mut exact_vertex_pts,
                    &mut edge_intersection_pts,
                );
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.point_data_mut().add_array(out_scalars.into());
    pd
}

fn contour_triangle(
    ids: [usize; 3],
    input: &PolyData,
    scalars: &[f64],
    isovalue: f64,
    out_points: &mut Points<f64>,
    out_lines: &mut CellArray,
    out_scalars: &mut DataArray<f64>,
    exact_vertex_pts: &mut HashMap<usize, usize>,
    edge_intersection_pts: &mut HashMap<(usize, usize), usize>,
) {
    const CASE_MASK: [usize; 3] = [1, 2, 4];
    const LINE_CASES: [[i32; 3]; 8] = [
        [-1, -1, -1],
        [0, 2, -1],
        [1, 0, -1],
        [1, 2, -1],
        [2, 1, -1],
        [0, 1, -1],
        [2, 0, -1],
        [-1, -1, -1],
    ];
    const EDGES: [[usize; 2]; 3] = [[0, 1], [1, 2], [2, 0]];

    let mut index = 0;
    for i in 0..3 {
        if scalars[ids[i]] >= isovalue {
            index |= CASE_MASK[i];
        }
    }

    insert_case_lines(
        ids,
        &LINE_CASES[index],
        &EDGES,
        input,
        scalars,
        isovalue,
        out_points,
        out_lines,
        out_scalars,
        exact_vertex_pts,
        edge_intersection_pts,
    );
}

fn contour_quad(
    ids: [usize; 4],
    input: &PolyData,
    scalars: &[f64],
    isovalue: f64,
    out_points: &mut Points<f64>,
    out_lines: &mut CellArray,
    out_scalars: &mut DataArray<f64>,
    exact_vertex_pts: &mut HashMap<usize, usize>,
    edge_intersection_pts: &mut HashMap<(usize, usize), usize>,
) {
    const CASE_MASK: [usize; 4] = [1, 2, 4, 8];
    const LINE_CASES: [[i32; 5]; 16] = [
        [-1, -1, -1, -1, -1],
        [0, 3, -1, -1, -1],
        [1, 0, -1, -1, -1],
        [1, 3, -1, -1, -1],
        [2, 1, -1, -1, -1],
        [0, 3, 2, 1, -1],
        [2, 0, -1, -1, -1],
        [2, 3, -1, -1, -1],
        [3, 2, -1, -1, -1],
        [0, 2, -1, -1, -1],
        [1, 0, 3, 2, -1],
        [1, 2, -1, -1, -1],
        [3, 1, -1, -1, -1],
        [0, 1, -1, -1, -1],
        [3, 0, -1, -1, -1],
        [-1, -1, -1, -1, -1],
    ];
    const EDGES: [[usize; 2]; 4] = [[0, 1], [1, 2], [3, 2], [0, 3]];

    let mut index = 0;
    for i in 0..4 {
        if scalars[ids[i]] >= isovalue {
            index |= CASE_MASK[i];
        }
    }

    insert_case_lines(
        ids,
        &LINE_CASES[index],
        &EDGES,
        input,
        scalars,
        isovalue,
        out_points,
        out_lines,
        out_scalars,
        exact_vertex_pts,
        edge_intersection_pts,
    );
}

fn insert_case_lines<const N: usize, const M: usize>(
    ids: [usize; N],
    case_edges: &[i32; M],
    edges: &[[usize; 2]],
    input: &PolyData,
    scalars: &[f64],
    isovalue: f64,
    out_points: &mut Points<f64>,
    out_lines: &mut CellArray,
    out_scalars: &mut DataArray<f64>,
    exact_vertex_pts: &mut HashMap<usize, usize>,
    edge_intersection_pts: &mut HashMap<(usize, usize), usize>,
) {
    let mut e = 0;
    while e + 1 < M && case_edges[e] >= 0 {
        let p0 = contour_edge_point(
            ids,
            edges[case_edges[e] as usize],
            input,
            scalars,
            isovalue,
            out_points,
            out_scalars,
            exact_vertex_pts,
            edge_intersection_pts,
        );
        let p1 = contour_edge_point(
            ids,
            edges[case_edges[e + 1] as usize],
            input,
            scalars,
            isovalue,
            out_points,
            out_scalars,
            exact_vertex_pts,
            edge_intersection_pts,
        );
        if p0 != p1 {
            out_lines.push_cell(&[p0 as i64, p1 as i64]);
        }
        e += 2;
    }
}

fn contour_edge_point<const N: usize>(
    ids: [usize; N],
    edge: [usize; 2],
    input: &PolyData,
    scalars: &[f64],
    isovalue: f64,
    out_points: &mut Points<f64>,
    out_scalars: &mut DataArray<f64>,
    exact_vertex_pts: &mut HashMap<usize, usize>,
    edge_intersection_pts: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let id0 = ids[edge[0]];
    let id1 = ids[edge[1]];
    let delta_scalar = scalars[id1] - scalars[id0];
    let (e1, e2, delta_scalar) = if delta_scalar > 0.0 {
        (id0, id1, delta_scalar)
    } else {
        (id1, id0, -delta_scalar)
    };

    let t = if delta_scalar == 0.0 {
        0.0
    } else {
        (isovalue - scalars[e1]) / delta_scalar
    };

    if t == 0.0 {
        return *exact_vertex_pts.entry(e1).or_insert_with(|| {
            let idx = out_points.len();
            out_points.push(input.points.get(e1));
            out_scalars.push_tuple(&[isovalue]);
            idx
        });
    }
    if t == 1.0 {
        return *exact_vertex_pts.entry(e2).or_insert_with(|| {
            let idx = out_points.len();
            out_points.push(input.points.get(e2));
            out_scalars.push_tuple(&[isovalue]);
            idx
        });
    }

    let edge_key = if e1 < e2 { (e1, e2) } else { (e2, e1) };
    *edge_intersection_pts.entry(edge_key).or_insert_with(|| {
        let p1 = input.points.get(e1);
        let p2 = input.points.get(e2);
        let idx = out_points.len();
        out_points.push([
            p1[0] + t * (p2[0] - p1[0]),
            p1[1] + t * (p2[1] - p1[1]),
            p1[2] + t * (p2[2] - p1[2]),
        ]);
        out_scalars.push_tuple(&[isovalue]);
        idx
    })
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
            0.0
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
    fn contour_range_single_uses_range_start() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = vec![0.0, 1.0, 0.5];
        let result = contour_range(&pd, &scalars, 0.25, 0.75, 1);

        assert_eq!(result.lines.num_cells(), 1);
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
