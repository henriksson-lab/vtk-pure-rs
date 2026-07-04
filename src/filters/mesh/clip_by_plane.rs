use crate::data::{CellArray, Points, PolyData};

/// Clip a PolyData mesh by a plane defined by a point and normal.
///
/// Keeps the half-space where `dot(p - point, normal) >= 0`.
/// Triangles that cross the plane are split, generating new vertices on the plane.
pub fn clip_by_plane(input: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let mut points = input.points.clone();
    let num_points = input.points.len();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();

    for cell in input.verts.iter() {
        if !cell_has_valid_points(cell, num_points) {
            continue;
        }
        if cell
            .iter()
            .any(|&id| signed_distance(input.points.get(id as usize), point, normal) >= 0.0)
        {
            verts.push_cell(cell);
        }
    }

    for cell in input.lines.iter() {
        if !cell_has_valid_points(cell, num_points) {
            continue;
        }
        clip_polyline(cell, &input.points, &mut points, point, normal, &mut lines);
    }

    for cell in input.polys.iter() {
        if !cell_has_valid_points(cell, num_points) {
            continue;
        }
        clip_polygon_cell(cell, &input.points, &mut points, point, normal, &mut polys);
    }

    for cell in input.strips.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell_has_valid_points(cell, num_points) {
            continue;
        }
        for i in 0..cell.len() - 2 {
            let tri = if i % 2 == 0 {
                [cell[i], cell[i + 1], cell[i + 2]]
            } else {
                [cell[i + 2], cell[i + 1], cell[i]]
            };
            clip_polygon_cell(&tri, &input.points, &mut points, point, normal, &mut polys);
        }
    }

    let mut output = PolyData::new();
    let mut used = vec![false; points.len()];
    for cell in verts.iter() {
        for &vid in cell {
            used[vid as usize] = true;
        }
    }
    for cell in lines.iter() {
        for &vid in cell {
            used[vid as usize] = true;
        }
    }
    for cell in polys.iter() {
        for &vid in cell {
            used[vid as usize] = true;
        }
    }

    let mut point_map = vec![0i64; points.len()];
    let mut compact_points = Points::new();
    for (i, is_used) in used.iter().enumerate() {
        if *is_used {
            point_map[i] = compact_points.len() as i64;
            compact_points.push(points.get(i));
        }
    }

    let mut compact_verts = CellArray::new();
    for cell in verts.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        compact_verts.push_cell(&remapped);
    }

    let mut compact_lines = CellArray::new();
    for cell in lines.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        compact_lines.push_cell(&remapped);
    }

    let mut compact_polys = CellArray::new();
    for cell in polys.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        compact_polys.push_cell(&remapped);
    }

    output.points = compact_points;
    output.verts = compact_verts;
    output.lines = compact_lines;
    output.polys = compact_polys;
    output
}

fn cell_has_valid_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn clip_polygon_cell(
    cell: &[i64],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point: [f64; 3],
    normal: [f64; 3],
    polys: &mut CellArray,
) {
    if cell.len() < 3 {
        return;
    }

    let dists: Vec<f64> = cell
        .iter()
        .map(|&id| signed_distance(src_points.get(id as usize), point, normal))
        .collect();

    let all_inside = dists.iter().all(|&d| d >= 0.0);
    let all_outside = dists.iter().all(|&d| d < 0.0);

    if all_inside {
        polys.push_cell(cell);
    } else if !all_outside {
        let clipped = clip_polygon(cell, &dists, src_points, all_points);
        if clipped.len() >= 3 {
            for i in 1..clipped.len() - 1 {
                polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
            }
        }
    }
}

fn clip_polyline(
    cell: &[i64],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
    point: [f64; 3],
    normal: [f64; 3],
    lines: &mut CellArray,
) {
    if cell.len() < 2 {
        return;
    }

    for i in 0..cell.len() - 1 {
        let vi = cell[i];
        let vj = cell[i + 1];
        let pi = src_points.get(vi as usize);
        let pj = src_points.get(vj as usize);
        let di = signed_distance(pi, point, normal);
        let dj = signed_distance(pj, point, normal);
        let inside_i = di >= 0.0;
        let inside_j = dj >= 0.0;

        match (inside_i, inside_j) {
            (true, true) => lines.push_cell(&[vi, vj]),
            (true, false) | (false, true) => {
                let t = di / (di - dj);
                let intersection = [
                    pi[0] + t * (pj[0] - pi[0]),
                    pi[1] + t * (pj[1] - pi[1]),
                    pi[2] + t * (pj[2] - pi[2]),
                ];
                let new_id = all_points.len() as i64;
                all_points.push(intersection);
                if inside_i {
                    lines.push_cell(&[vi, new_id]);
                } else {
                    lines.push_cell(&[new_id, vj]);
                }
            }
            (false, false) => {}
        }
    }
}

fn signed_distance(p: [f64; 3], point: [f64; 3], normal: [f64; 3]) -> f64 {
    (p[0] - point[0]) * normal[0] + (p[1] - point[1]) * normal[1] + (p[2] - point[2]) * normal[2]
}

/// Clip a single polygon by the plane, returning vertex indices of the clipped result.
fn clip_polygon(
    cell: &[i64],
    dists: &[f64],
    src_points: &Points<f64>,
    all_points: &mut Points<f64>,
) -> Vec<i64> {
    let n = cell.len();
    let mut result: Vec<i64> = Vec::new();

    for i in 0..n {
        let j = (i + 1) % n;
        let di: f64 = dists[i];
        let dj: f64 = dists[j];
        let vi = cell[i];
        let vj = cell[j];

        if di >= 0.0 {
            result.push(vi);
        }

        // If edge crosses the plane, add intersection point
        if (di >= 0.0) != (dj >= 0.0) {
            let t: f64 = di / (di - dj);
            let pi = src_points.get(vi as usize);
            let pj = src_points.get(vj as usize);
            let intersection: [f64; 3] = [
                pi[0] + t * (pj[0] - pi[0]),
                pi[1] + t * (pj[1] - pi[1]),
                pi[2] + t * (pj[2] - pi[2]),
            ];
            let new_id: i64 = all_points.len() as i64;
            all_points.push(intersection);
            result.push(new_id);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_fully_inside() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Plane at origin, normal +x => everything with x >= 0 kept
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn triangle_fully_outside() {
        let pd = PolyData::from_triangles(
            vec![[-3.0, 0.0, 0.0], [-2.0, 0.0, 0.0], [-2.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn triangle_split_by_plane() {
        // Triangle straddling x=0 plane
        let pd = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        // Should have at least one triangle
        assert!(result.polys.num_cells() >= 1);
        // Check that all vertices referenced by cells are on the positive side
        for cell in result.polys.iter() {
            for &id in cell {
                let p = result.points.get(id as usize);
                assert!(p[0] >= -1e-10, "cell vertex {} has x={}", id, p[0]);
            }
        }
    }

    #[test]
    fn line_cells_are_clipped() {
        let pd = PolyData::from_lines(vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], vec![[0, 1]]);
        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.lines.num_cells(), 1);
        for cell in result.lines.iter() {
            for &id in cell {
                let p = result.points.get(id as usize);
                assert!(p[0] >= -1e-10, "line vertex {} has x={}", id, p[0]);
            }
        }
    }

    #[test]
    fn strips_are_clipped_as_triangles() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = clip_by_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[3, 2, 1]);
    }
}
