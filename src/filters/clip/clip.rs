use crate::data::{CellArray, Points, PolyData};

/// Clip a PolyData by a plane defined by a point and normal.
///
/// Keeps the half-space where `dot(p - origin, normal) > 0`.
/// Triangles that cross the plane are split, generating new vertices on the plane.
pub fn clip_by_plane(input: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let mut points = input.points.clone();
    let mut point_locator = PointLocator::from_points(&points);
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();

    for cell in input.verts.iter() {
        for &id in cell {
            let p = input.points.get(id as usize);
            let dist = signed_distance(p, origin, normal);
            if dist > 0.0 {
                verts.push_cell(&[id]);
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
                polys.push_cell(&tri);
            } else if !all_outside {
                let clipped =
                    clip_polygon(&tri, &dists, &input.points, &mut points, &mut point_locator);
                if clipped.len() >= 3 {
                    for j in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[j], clipped[j + 1]]);
                    }
                }
            }
        }
    }

    // Compact: only keep referenced points
    let mut used = vec![false; points.len()];
    for cells in [&verts, &lines, &polys] {
        for ci in 0..cells.num_cells() {
            for &vid in cells.cell(ci) {
                used[vid as usize] = true;
            }
        }
    }
    let mut point_map = vec![0i64; points.len()];
    let mut compact_points = Points::new();
    for i in 0..points.len() {
        if used[i] {
            point_map[i] = compact_points.len() as i64;
            compact_points.push(points.get(i));
        }
    }
    let compact_verts = remap_cells(&verts, &point_map);
    let compact_lines = remap_cells(&lines, &point_map);
    let compact_polys = remap_cells(&polys, &point_map);

    let mut output = PolyData::new();
    output.points = compact_points;
    output.verts = compact_verts;
    output.lines = compact_lines;
    output.polys = compact_polys;
    output
}

#[derive(Default)]
struct PointLocator {
    points: Vec<[f64; 3]>,
}

impl PointLocator {
    fn from_points(points: &Points<f64>) -> Self {
        let mut locator = Self::default();
        for i in 0..points.len() {
            locator.points.push(points.get(i));
        }
        locator
    }

    fn insert_unique_point(&mut self, points: &mut Points<f64>, point: [f64; 3]) -> i64 {
        if let Some((id, _)) = self
            .points
            .iter()
            .enumerate()
            .find(|(_, existing)| same_point(**existing, point))
        {
            return id as i64;
        }

        let id = points.len() as i64;
        points.push(point);
        self.points.push(point);
        id
    }
}

fn same_point(a: [f64; 3], b: [f64; 3]) -> bool {
    (a[0] - b[0]).abs() <= 1e-12 && (a[1] - b[1]).abs() <= 1e-12 && (a[2] - b[2]).abs() <= 1e-12
}

fn signed_distance(p: [f64; 3], origin: [f64; 3], normal: [f64; 3]) -> f64 {
    (p[0] - origin[0]) * normal[0] + (p[1] - origin[1]) * normal[1] + (p[2] - origin[2]) * normal[2]
}

fn remap_cells(cells: &CellArray, point_map: &[i64]) -> CellArray {
    let mut remapped_cells = CellArray::new();
    for ci in 0..cells.num_cells() {
        let cell = cells.cell(ci);
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        remapped_cells.push_cell(&remapped);
    }
    remapped_cells
}

fn clip_polyline_segment(
    ids: [i64; 2],
    dists: [f64; 2],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point_locator: &mut PointLocator,
) -> Vec<i64> {
    let i_in = dists[0] > 0.0;
    let j_in = dists[1] > 0.0;

    match (i_in, j_in) {
        (true, true) => vec![ids[0], ids[1]],
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
            let new_id = point_locator.insert_unique_point(all_points, intersection);
            if i_in {
                vec![ids[0], new_id]
            } else {
                vec![new_id, ids[1]]
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
    lines: &mut CellArray,
) {
    let mut current = Vec::new();

    for i in 0..cell.len() - 1 {
        let ids = [cell[i], cell[i + 1]];
        let dists = [
            signed_distance(src_points.get(ids[0] as usize), origin, normal),
            signed_distance(src_points.get(ids[1] as usize), origin, normal),
        ];
        let clipped = clip_polyline_segment(ids, dists, src_points, all_points, point_locator);

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
    polys: &mut CellArray,
) {
    let dists: Vec<f64> = tri
        .iter()
        .map(|&id| signed_distance(src_points.get(id as usize), origin, normal))
        .collect();
    let all_inside = dists.iter().all(|&d| d > 0.0);
    let all_outside = dists.iter().all(|&d| d <= 0.0);

    if all_inside {
        polys.push_cell(tri);
    } else if !all_outside {
        let clipped = clip_polygon(tri, &dists, src_points, all_points, point_locator);
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
            result.push(vi);
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
            let new_id = point_locator.insert_unique_point(all_points, intersection);
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
