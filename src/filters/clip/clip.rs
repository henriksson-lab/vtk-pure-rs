use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Clip a PolyData by a plane defined by a point and normal.
///
/// Keeps the half-space where `dot(p - origin, normal) > 0`.
/// Triangles that cross the plane are split, generating new vertices on the plane.
pub fn clip_by_plane(input: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let mut points = Points::new();
    let mut point_locator = PointLocator::default();
    let mut point_map = vec![-1i64; input.points.len()];
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();

    for cell in input.verts.iter() {
        for &id in cell {
            let p = input.points.get(id as usize);
            let dist = signed_distance(p, origin, normal);
            if dist > 0.0 {
                let out_id =
                    get_or_copy_input_point(id, &input.points, &mut points, &mut point_map);
                verts.push_cell(&[out_id]);
            }
        }
    }

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        clip_polyline(
            cell,
            origin,
            normal,
            &input.points,
            &mut points,
            &mut point_locator,
            &mut point_map,
            &mut lines,
        );
    }

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        for i in 1..cell.len() - 1 {
            let tri = [cell[0], cell[i], cell[i + 1]];
            clip_triangle_by_plane(
                &tri,
                origin,
                normal,
                &input.points,
                &mut points,
                &mut point_locator,
                &mut point_map,
                &mut polys,
            );
        }
    }

    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 2], strip[i + 1], strip[i]]
            };
            let dists: Vec<f64> = tri
                .iter()
                .map(|&id| signed_distance(input.points.get(id as usize), origin, normal))
                .collect();
            let all_inside = dists.iter().all(|&d| d > 0.0);
            let all_outside = dists.iter().all(|&d| d <= 0.0);

            if all_inside {
                let mapped = [
                    get_or_copy_input_point(tri[0], &input.points, &mut points, &mut point_map),
                    get_or_copy_input_point(tri[1], &input.points, &mut points, &mut point_map),
                    get_or_copy_input_point(tri[2], &input.points, &mut points, &mut point_map),
                ];
                polys.push_cell(&mapped);
            } else if !all_outside {
                let clipped = clip_polygon(
                    &tri,
                    &dists,
                    &input.points,
                    &mut points,
                    &mut point_locator,
                    &mut point_map,
                );
                if clipped.len() >= 3 {
                    for j in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[j], clipped[j + 1]]);
                    }
                }
            }
        }
    }

    let mut output = PolyData::new();
    output.points = points;
    output.verts = verts;
    output.lines = lines;
    output.polys = polys;
    output
}

fn get_or_copy_input_point(
    input_id: i64,
    src_points: &Points<f64>,
    out_points: &mut Points<f64>,
    point_map: &mut [i64],
) -> i64 {
    let input_idx = input_id as usize;
    let out_id = point_map[input_idx];
    if out_id >= 0 {
        return out_id;
    }

    let out_id = out_points.len() as i64;
    out_points.push(src_points.get(input_idx));
    point_map[input_idx] = out_id;
    out_id
}

#[derive(Default)]
struct PointLocator {
    edge_points: HashMap<(i64, i64), i64>,
}

impl PointLocator {
    fn insert_edge_point(
        &mut self,
        points: &mut Points<f64>,
        edge: [i64; 2],
        point: [f64; 3],
    ) -> i64 {
        let key = if edge[0] <= edge[1] {
            (edge[0], edge[1])
        } else {
            (edge[1], edge[0])
        };
        if let Some(&id) = self.edge_points.get(&key) {
            return id;
        }

        let id = points.len() as i64;
        points.push(point);
        self.edge_points.insert(key, id);
        id
    }
}

fn signed_distance(p: [f64; 3], origin: [f64; 3], normal: [f64; 3]) -> f64 {
    (p[0] - origin[0]) * normal[0] + (p[1] - origin[1]) * normal[1] + (p[2] - origin[2]) * normal[2]
}

fn clip_polyline_segment(
    ids: [i64; 2],
    dists: [f64; 2],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point_locator: &mut PointLocator,
    point_map: &mut [i64],
) -> Vec<i64> {
    let i_in = dists[0] > 0.0;
    let j_in = dists[1] > 0.0;

    match (i_in, j_in) {
        (true, true) => vec![
            get_or_copy_input_point(ids[0], src_points, all_points, point_map),
            get_or_copy_input_point(ids[1], src_points, all_points, point_map),
        ],
        (false, false) => Vec::new(),
        _ => {
            let t = dists[0] / (dists[0] - dists[1]);
            let pi = src_points.get(ids[0] as usize);
            let pj = src_points.get(ids[1] as usize);
            let intersection = [
                pi[0] + t * (pj[0] - pi[0]),
                pi[1] + t * (pj[1] - pi[1]),
                pi[2] + t * (pj[2] - pi[2]),
            ];
            let new_id = point_locator.insert_edge_point(all_points, ids, intersection);
            if i_in {
                vec![
                    get_or_copy_input_point(ids[0], src_points, all_points, point_map),
                    new_id,
                ]
            } else {
                vec![
                    new_id,
                    get_or_copy_input_point(ids[1], src_points, all_points, point_map),
                ]
            }
        }
    }
}

fn clip_polyline(
    cell: &[i64],
    origin: [f64; 3],
    normal: [f64; 3],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point_locator: &mut PointLocator,
    point_map: &mut [i64],
    lines: &mut CellArray,
) {
    let mut current = Vec::new();

    for i in 0..cell.len() - 1 {
        let ids = [cell[i], cell[i + 1]];
        let dists = [
            signed_distance(src_points.get(ids[0] as usize), origin, normal),
            signed_distance(src_points.get(ids[1] as usize), origin, normal),
        ];
        let clipped =
            clip_polyline_segment(ids, dists, src_points, all_points, point_locator, point_map);

        if clipped.len() == 2 {
            if current.is_empty() {
                current.extend_from_slice(&clipped);
            } else if current.last() == Some(&clipped[0]) {
                current.push(clipped[1]);
            } else {
                if current.len() >= 2 {
                    lines.push_cell(&current);
                }
                current.clear();
                current.extend_from_slice(&clipped);
            }
        } else if current.len() >= 2 {
            lines.push_cell(&current);
            current.clear();
        }
    }

    if current.len() >= 2 {
        lines.push_cell(&current);
    }
}

fn clip_triangle_by_plane(
    tri: &[i64; 3],
    origin: [f64; 3],
    normal: [f64; 3],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point_locator: &mut PointLocator,
    point_map: &mut [i64],
    polys: &mut CellArray,
) {
    let dists: Vec<f64> = tri
        .iter()
        .map(|&id| signed_distance(src_points.get(id as usize), origin, normal))
        .collect();
    let all_inside = dists.iter().all(|&d| d > 0.0);
    let all_outside = dists.iter().all(|&d| d <= 0.0);

    if all_inside {
        let mapped = [
            get_or_copy_input_point(tri[0], src_points, all_points, point_map),
            get_or_copy_input_point(tri[1], src_points, all_points, point_map),
            get_or_copy_input_point(tri[2], src_points, all_points, point_map),
        ];
        polys.push_cell(&mapped);
    } else if !all_outside {
        let clipped = clip_polygon(
            tri,
            &dists,
            src_points,
            all_points,
            point_locator,
            point_map,
        );
        if clipped.len() >= 3 {
            for i in 1..clipped.len() - 1 {
                polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
            }
        }
    }
}

/// Clip a single polygon, returning new vertex indices for the clipped result.
fn clip_polygon(
    cell: &[i64],
    dists: &[f64],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point_locator: &mut PointLocator,
    point_map: &mut [i64],
) -> Vec<i64> {
    let n = cell.len();
    let mut result = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        let di = dists[i];
        let dj = dists[j];
        let vi = cell[i];
        let vj = cell[j];

        if di > 0.0 {
            result.push(get_or_copy_input_point(
                vi, src_points, all_points, point_map,
            ));
        }

        // If edge crosses the plane, add intersection point
        if (di > 0.0) != (dj > 0.0) {
            let t = di / (di - dj);
            let pi = src_points.get(vi as usize);
            let pj = src_points.get(vj as usize);
            let intersection = [
                pi[0] + t * (pj[0] - pi[0]),
                pi[1] + t * (pj[1] - pi[1]),
                pi[2] + t * (pj[2] - pi[2]),
            ];
            let new_id = point_locator.insert_edge_point(all_points, [vi, vj], intersection);
            result.push(new_id);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_triangle_keeps_inside() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // Plane at origin with +Z normal (everything is on the plane)
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        // VTK's normal clip sense keeps values strictly greater than 0.
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn clip_removes_outside() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 1.0, -1.0]],
            vec![[0, 1, 2]],
        );

        // Plane at origin, normal +Z → triangle is entirely below
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn clip_splits_triangle() {
        let pd = PolyData::from_triangles(
            vec![
                [-1.0, 0.0, 0.0], // inside (x < 0 → outside if normal is +X)
                [1.0, 0.0, 0.0],  // inside
                [1.0, 1.0, 0.0],  // inside
            ],
            vec![[0, 1, 2]],
        );

        // Clip by x=0 plane, keeping x >= 0
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        // Point 0 is outside (x=-1), points 1,2 are inside
        // Should create a clipped polygon → triangulated
        assert!(result.polys.num_cells() >= 1);
        // All resulting points should have x >= -1e-6
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            if i >= 3 {
                // New intersection points should be on the plane
                assert!(p[0].abs() < 1e-10, "intersection point x={}", p[0]);
            }
        }
    }

    #[test]
    fn clip_polyline_keeps_contiguous_segments_as_one_cell() {
        let pd = PolyData::from_polyline(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);

        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.lines.cell(0).len(), 3);
    }

    #[test]
    fn clip_strip_uses_vtk_odd_triangle_order() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[3, 2, 1]);
    }
}
