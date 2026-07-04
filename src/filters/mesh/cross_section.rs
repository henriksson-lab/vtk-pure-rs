//! Cross-section analysis: compute area, centroid, and moments of
//! cross-sections through a mesh at multiple positions along an axis.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Compute cross-section areas along an axis at regular intervals.
///
/// Slices the mesh at `n_slices` positions along the given axis
/// and returns a Table with columns "Position" and "Area".
pub fn cross_section_profile(
    mesh: &PolyData,
    axis: usize, // 0=X, 1=Y, 2=Z
    n_slices: usize,
) -> Table {
    if axis >= 3 || n_slices == 0 {
        return Table::new();
    }

    let Some((min_v, max_v)) = axis_range(mesh, axis) else {
        return Table::new();
    };
    let range = max_v - min_v;
    if range < 1e-15 {
        return Table::new();
    }

    let mut positions = Vec::with_capacity(n_slices);
    let mut areas = Vec::with_capacity(n_slices);

    for si in 0..n_slices {
        let t = (si as f64 + 0.5) / n_slices as f64;
        let pos = min_v + t * range;
        positions.push(pos);
        areas.push(section_area(mesh, axis, pos));
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "Position", positions, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Area", areas, 1)))
}

/// Compute the volume of a mesh by integrating cross-section areas.
pub fn volume_from_cross_sections(mesh: &PolyData, axis: usize, n_slices: usize) -> f64 {
    let profile = cross_section_profile(mesh, axis, n_slices);
    if profile.num_rows() == 0 {
        return 0.0;
    }

    let Some((min_v, max_v)) = axis_range(mesh, axis) else {
        return 0.0;
    };
    let dz = (max_v - min_v) / n_slices as f64;
    let n = profile.num_rows();
    let mut vol = 0.0;
    for i in 0..n {
        if let Some(area) = profile.value_f64(i, "Area") {
            vol += area * dz;
        }
    }
    vol
}

fn axis_range(mesh: &PolyData, axis: usize) -> Option<(f64, f64)> {
    if mesh.points.len() == 0 || axis >= 3 {
        return None;
    }

    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        min_v = min_v.min(p[axis]);
        max_v = max_v.max(p[axis]);
    }
    Some((min_v, max_v))
}

fn section_area(mesh: &PolyData, axis: usize, position: f64) -> f64 {
    let mut segments = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 1..cell.len() - 1 {
            let tri = [cell[0], cell[i], cell[i + 1]];
            if !valid_triangle_ids(&tri, mesh.points.len()) {
                continue;
            }
            if let Some(segment) = intersect_triangle(mesh, axis, position, tri) {
                segments.push(segment);
            }
        }
    }
    loops_area(&segments)
}

fn valid_triangle_ids(tri: &[i64; 3], n_points: usize) -> bool {
    tri.iter().all(|&id| id >= 0 && (id as usize) < n_points)
}

fn intersect_triangle(
    mesh: &PolyData,
    axis: usize,
    position: f64,
    tri: [i64; 3],
) -> Option<([f64; 2], [f64; 2])> {
    const EPS: f64 = 1e-12;

    let points = [
        mesh.points.get(tri[0] as usize),
        mesh.points.get(tri[1] as usize),
        mesh.points.get(tri[2] as usize),
    ];
    let dist = [
        points[0][axis] - position,
        points[1][axis] - position,
        points[2][axis] - position,
    ];
    if dist.iter().all(|&d| d > EPS) || dist.iter().all(|&d| d < -EPS) {
        return None;
    }

    let mut hits = Vec::new();
    for i in 0..3 {
        let j = (i + 1) % 3;
        let di = dist[i];
        let dj = dist[j];

        if di.abs() <= EPS && dj.abs() <= EPS {
            push_unique_point(&mut hits, project(points[i], axis));
            push_unique_point(&mut hits, project(points[j], axis));
        } else if di.abs() <= EPS {
            push_unique_point(&mut hits, project(points[i], axis));
        } else if dj.abs() <= EPS {
            push_unique_point(&mut hits, project(points[j], axis));
        } else if di * dj < 0.0 {
            let t = (position - points[i][axis]) / (points[j][axis] - points[i][axis]);
            let p = [
                points[i][0] + t * (points[j][0] - points[i][0]),
                points[i][1] + t * (points[j][1] - points[i][1]),
                points[i][2] + t * (points[j][2] - points[i][2]),
            ];
            push_unique_point(&mut hits, project(p, axis));
        }
    }

    if hits.len() < 2 {
        return None;
    }

    let mut best = (hits[0], hits[1]);
    let mut best_d2 = distance2(hits[0], hits[1]);
    for i in 0..hits.len() {
        for j in i + 1..hits.len() {
            let d2 = distance2(hits[i], hits[j]);
            if d2 > best_d2 {
                best = (hits[i], hits[j]);
                best_d2 = d2;
            }
        }
    }
    if best_d2 <= EPS * EPS {
        None
    } else {
        Some(best)
    }
}

fn project(p: [f64; 3], axis: usize) -> [f64; 2] {
    match axis {
        0 => [p[1], p[2]],
        1 => [p[0], p[2]],
        _ => [p[0], p[1]],
    }
}

fn push_unique_point(points: &mut Vec<[f64; 2]>, point: [f64; 2]) {
    if !points.iter().any(|&p| distance2(p, point) <= 1e-24) {
        points.push(point);
    }
}

fn loops_area(segments: &[([f64; 2], [f64; 2])]) -> f64 {
    let mut points: Vec<[f64; 2]> = Vec::new();
    let mut edges = std::collections::HashSet::new();

    for &(a, b) in segments {
        if distance2(a, b) <= 1e-24 {
            continue;
        }
        let ia = point_index(&mut points, a);
        let ib = point_index(&mut points, b);
        let edge = if ia < ib { (ia, ib) } else { (ib, ia) };
        edges.insert(edge);
    }

    let mut adjacency = vec![Vec::new(); points.len()];
    for &(a, b) in &edges {
        adjacency[a].push(b);
        adjacency[b].push(a);
    }

    let mut visited = std::collections::HashSet::new();
    let mut area = 0.0;
    for &(a, b) in &edges {
        let edge = if a < b { (a, b) } else { (b, a) };
        if visited.contains(&edge) {
            continue;
        }
        let mut loop_points = vec![a];
        let mut prev = a;
        let mut cur = b;
        visited.insert(edge);

        loop {
            loop_points.push(cur);
            if cur == a {
                break;
            }
            let next = adjacency[cur].iter().copied().find(|&candidate| {
                candidate != prev && !visited.contains(&ordered_edge(cur, candidate))
            });
            let Some(next) = next else {
                break;
            };
            visited.insert(ordered_edge(cur, next));
            prev = cur;
            cur = next;
        }

        if loop_points.len() >= 4 && loop_points.last() == Some(&a) {
            area += polygon_area(&loop_points[..loop_points.len() - 1], &points).abs();
        }
    }
    area
}

fn point_index(points: &mut Vec<[f64; 2]>, point: [f64; 2]) -> usize {
    if let Some(idx) = points.iter().position(|&p| distance2(p, point) <= 1e-18) {
        idx
    } else {
        let idx = points.len();
        points.push(point);
        idx
    }
}

fn ordered_edge(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn polygon_area(loop_points: &[usize], points: &[[f64; 2]]) -> f64 {
    let mut area = 0.0;
    for i in 0..loop_points.len() {
        let a = points[loop_points[i]];
        let b = points[loop_points[(i + 1) % loop_points.len()]];
        area += a[0] * b[1] - b[0] * a[1];
    }
    0.5 * area
}

fn distance2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_z() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let profile = cross_section_profile(&mesh, 2, 10);
        assert_eq!(profile.num_rows(), 10);
        assert!(profile.column_by_name("Position").is_some());
        assert!(profile.column_by_name("Area").is_some());
    }

    #[test]
    fn volume_sphere() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let vol = volume_from_cross_sections(&mesh, 2, 50);
        assert!(vol > 0.0);
    }

    #[test]
    fn empty() {
        let profile = cross_section_profile(&PolyData::new(), 0, 10);
        assert_eq!(profile.num_rows(), 0);
    }

    #[test]
    fn cube_sections_have_unit_area() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [1, 2, 6],
                [1, 6, 5],
                [2, 3, 7],
                [2, 7, 6],
                [3, 0, 4],
                [3, 4, 7],
            ],
        );

        let profile = cross_section_profile(&mesh, 2, 3);
        for i in 0..profile.num_rows() {
            let area = profile.value_f64(i, "Area").unwrap();
            assert!((area - 1.0).abs() < 1e-10);
        }

        let volume = volume_from_cross_sections(&mesh, 2, 3);
        assert!((volume - 1.0).abs() < 1e-10);
    }

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 1.0]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.polys.push_cell(&[0, 1, -1]);
        mesh.polys.push_cell(&[0, 1, 99]);

        let profile = cross_section_profile(&mesh, 2, 2);
        assert_eq!(profile.num_rows(), 2);
        for i in 0..profile.num_rows() {
            assert!(profile.value_f64(i, "Area").unwrap().is_finite());
        }
    }
}
