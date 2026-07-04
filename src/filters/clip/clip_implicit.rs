use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};
use crate::types::ImplicitFunction;

/// Clip a PolyData mesh with an implicit function.
///
/// Keeps the region where `f(point) > 0`, matching vtkClipPolyData's
/// default InsideOut=off sense for Value=0. Polygons crossing the implicit
/// boundary are split by linear interpolation.
pub fn clip_with_implicit(input: &PolyData, func: &dyn ImplicitFunction) -> PolyData {
    let n = input.points.len();
    let values: Vec<f64> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            func.evaluate(p[0], p[1], p[2])
        })
        .collect();

    let mut points = input.points.clone();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut edge_locator = HashMap::new();

    for cell in input.verts.iter() {
        if cell.iter().any(|&id| id < 0 || id as usize >= n) {
            continue;
        }
        let kept: Vec<i64> = cell
            .iter()
            .copied()
            .filter(|&id| values[id as usize] > 0.0)
            .collect();
        if !kept.is_empty() {
            verts.push_cell(&kept);
        }
    }

    for cell in input.lines.iter() {
        if cell.iter().any(|&id| id < 0 || id as usize >= n) {
            continue;
        }
        clip_polyline(
            cell,
            &values,
            &input.points,
            &mut points,
            &mut edge_locator,
            &mut lines,
        );
    }

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if cell.iter().any(|&id| id < 0 || id as usize >= n) {
            continue;
        }

        let scalars: Vec<f64> = cell.iter().map(|&id| values[id as usize]).collect();
        let all_inside = scalars.iter().all(|&s| s > 0.0);
        let all_outside = scalars.iter().all(|&s| s <= 0.0);

        if all_inside {
            polys.push_cell(cell);
        } else if !all_outside {
            let clipped = clip_polygon(
                cell,
                &scalars,
                &input.points,
                &mut points,
                &mut edge_locator,
            );
            if clipped.len() >= 3 {
                for i in 1..clipped.len() - 1 {
                    polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                }
            }
        }
    }

    for cell in input.strips.iter() {
        if cell.len() < 3 || cell.iter().any(|&id| id < 0 || id as usize >= n) {
            continue;
        }
        for i in 0..cell.len() - 2 {
            let tri = if i % 2 == 0 {
                [cell[i], cell[i + 1], cell[i + 2]]
            } else {
                [cell[i + 1], cell[i], cell[i + 2]]
            };
            let scalars = [
                values[tri[0] as usize],
                values[tri[1] as usize],
                values[tri[2] as usize],
            ];
            let all_inside = scalars.iter().all(|&s| s > 0.0);
            let all_outside = scalars.iter().all(|&s| s <= 0.0);

            if all_inside {
                polys.push_cell(&tri);
            } else if !all_outside {
                let clipped = clip_polygon(
                    &tri,
                    &scalars,
                    &input.points,
                    &mut points,
                    &mut edge_locator,
                );
                if clipped.len() >= 3 {
                    for j in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[j], clipped[j + 1]]);
                    }
                }
            }
        }
    }

    compact_poly_data(points, verts, lines, polys)
}

fn clip_polygon(
    ids: &[i64],
    scalars: &[f64],
    input_points: &Points<f64>,
    points: &mut Points<f64>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
) -> Vec<i64> {
    let mut result = Vec::new();

    for i in 0..ids.len() {
        let j = (i + 1) % ids.len();
        let si = scalars[i];
        let sj = scalars[j];

        if si > 0.0 {
            result.push(ids[i]);
        }

        if (si > 0.0) != (sj > 0.0) {
            let ds = sj - si;
            if ds.abs() > 1e-15 {
                let edge_key = if ids[i] < ids[j] {
                    (ids[i], ids[j])
                } else {
                    (ids[j], ids[i])
                };
                let id = if let Some(&id) = edge_locator.get(&edge_key) {
                    id
                } else {
                    let t = (-si / ds).clamp(0.0, 1.0);
                    let pi = input_points.get(ids[i] as usize);
                    let pj = input_points.get(ids[j] as usize);
                    let p = [
                        pi[0] + t * (pj[0] - pi[0]),
                        pi[1] + t * (pj[1] - pi[1]),
                        pi[2] + t * (pj[2] - pi[2]),
                    ];
                    let id = points.len() as i64;
                    points.push(p);
                    edge_locator.insert(edge_key, id);
                    id
                };
                result.push(id);
            }
        }
    }

    result
}

fn clip_polyline(
    ids: &[i64],
    scalars: &[f64],
    input_points: &Points<f64>,
    points: &mut Points<f64>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    lines: &mut CellArray,
) {
    if ids.len() < 2 {
        return;
    }

    for i in 0..ids.len() - 1 {
        let vi = ids[i];
        let vj = ids[i + 1];
        let si = scalars[vi as usize];
        let sj = scalars[vj as usize];
        let inside_i = si > 0.0;
        let inside_j = sj > 0.0;

        match (inside_i, inside_j) {
            (true, true) => lines.push_cell(&[vi, vj]),
            (true, false) | (false, true) => {
                let ds = sj - si;
                if ds.abs() <= 1e-15 {
                    continue;
                }
                let edge_key = if vi < vj { (vi, vj) } else { (vj, vi) };
                let id = if let Some(&id) = edge_locator.get(&edge_key) {
                    id
                } else {
                    let t = (-si / ds).clamp(0.0, 1.0);
                    let pi = input_points.get(vi as usize);
                    let pj = input_points.get(vj as usize);
                    let p = [
                        pi[0] + t * (pj[0] - pi[0]),
                        pi[1] + t * (pj[1] - pi[1]),
                        pi[2] + t * (pj[2] - pi[2]),
                    ];
                    let id = points.len() as i64;
                    points.push(p);
                    edge_locator.insert(edge_key, id);
                    id
                };
                if inside_i {
                    lines.push_cell(&[vi, id]);
                } else {
                    lines.push_cell(&[id, vj]);
                }
            }
            (false, false) => {}
        }
    }
}

fn compact_poly_data(
    points: Points<f64>,
    verts: CellArray,
    lines: CellArray,
    polys: CellArray,
) -> PolyData {
    let mut used = vec![false; points.len()];
    for cell in verts.iter() {
        for &id in cell {
            used[id as usize] = true;
        }
    }
    for cell in lines.iter() {
        for &id in cell {
            used[id as usize] = true;
        }
    }
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

    let mut compact_verts = CellArray::new();
    for cell in verts.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        compact_verts.push_cell(&remapped);
    }

    let mut compact_lines = CellArray::new();
    for cell in lines.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        compact_lines.push_cell(&remapped);
    }

    let mut compact_polys = CellArray::new();
    for cell in polys.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        compact_polys.push_cell(&remapped);
    }

    let mut result = PolyData::new();
    result.points = compact_points;
    result.verts = compact_verts;
    result.lines = compact_lines;
    result.polys = compact_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ImplicitPlane, ImplicitSphere};

    #[test]
    fn clip_with_plane() {
        // Clip a quad at x=0.5 with a plane at x=0
        let pd = PolyData::from_triangles(
            vec![
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        // Plane at x=1.5, normal pointing +X; vtkClipPolyData's default
        // InsideOut=off keeps f > 0, i.e. x > 1.5.
        let plane = ImplicitPlane::new([1.5, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let clipped = clip_with_implicit(&pd, &plane);
        assert_eq!(clipped.polys.num_cells(), 2);
        for i in 0..clipped.points.len() {
            assert!(clipped.points.get(i)[0] >= 1.5 - 1e-10);
        }
    }

    #[test]
    fn clip_with_sphere() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0], // inside
                [5.0, 5.0, 5.0],
                [6.0, 5.0, 5.0],
                [5.0, 6.0, 5.0],
            ], // outside
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 1.0);
        let clipped = clip_with_implicit(&pd, &sphere);
        assert_eq!(clipped.polys.num_cells(), 1);
        for i in 0..clipped.points.len() {
            let p = clipped.points.get(i);
            assert!(p[0] * p[0] + p[1] * p[1] + p[2] * p[2] >= 1.0 - 1e-10);
        }
    }

    #[test]
    fn clip_keeps_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 10.0);
        let clipped = clip_with_implicit(&pd, &sphere);
        assert_eq!(clipped.polys.num_cells(), 0);
    }

    #[test]
    fn clips_line_cells() {
        let pd = PolyData::from_lines(vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], vec![[0, 1]]);
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let clipped = clip_with_implicit(&pd, &plane);
        assert_eq!(clipped.lines.num_cells(), 1);
        for cell in clipped.lines.iter() {
            for &id in cell {
                assert!(clipped.points.get(id as usize)[0] >= -1e-10);
            }
        }
    }

    #[test]
    fn clips_triangle_strips_to_polys() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let clipped = clip_with_implicit(&pd, &plane);
        assert!(clipped.polys.num_cells() > 0);
        assert_eq!(clipped.strips.num_cells(), 0);
        for cell in clipped.polys.iter() {
            for &id in cell {
                assert!(clipped.points.get(id as usize)[0] >= -1e-10);
            }
        }
    }
}
