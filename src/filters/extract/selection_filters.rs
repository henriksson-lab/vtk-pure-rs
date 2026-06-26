//! Selection filters: spatial cell/point selection methods.
//!
//! - CellDistanceSelector: select cells within distance of seed cells
//! - KdTreeSelector: spatial selection via k-d tree radius/box queries
//! - LinearSelector: select cells along a line/ray

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Select cells within a topological distance from seed cells.
///
/// Uses BFS on cell adjacency (shared vertices). Returns a PolyData with
/// a "SelectionDistance" cell data array containing the hop distance.
pub fn cell_distance_selector(
    input: &PolyData,
    seed_cells: &[usize],
    max_distance: usize,
) -> PolyData {
    let num_cells = input.polys.num_cells();

    // Build cell adjacency via shared points
    let mut point_to_cells: Vec<Vec<usize>> = vec![Vec::new(); input.points.len()];
    for ci in 0..num_cells {
        let cell = input.polys.cell(ci);
        for &vid in cell {
            point_to_cells[vid as usize].push(ci);
        }
    }

    let mut distances = vec![usize::MAX; num_cells];
    let mut queue = std::collections::VecDeque::new();

    for &seed in seed_cells {
        if seed < num_cells && distances[seed] == usize::MAX {
            distances[seed] = 0;
            queue.push_back(seed);
        }
    }

    while let Some(ci) = queue.pop_front() {
        let d = distances[ci];
        if d >= max_distance {
            continue;
        }
        let cell: Vec<i64> = input.polys.cell(ci).to_vec();
        for vid in &cell {
            for &neighbor in &point_to_cells[*vid as usize] {
                if distances[neighbor] == usize::MAX {
                    distances[neighbor] = d + 1;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // Extract selected cells
    let mut selected_cells: Vec<usize> = Vec::new();
    let mut dist_values: Vec<f64> = Vec::new();
    for (ci, &d) in distances.iter().enumerate() {
        if d <= max_distance {
            selected_cells.push(ci);
            dist_values.push(d as f64);
        }
    }

    extract_cells_by_indices(input, &selected_cells, "SelectionDistance", &dist_values)
}

/// Select points within a radius of a query point using brute-force search.
///
/// Returns a PolyData with only the selected points (as vertex cells).
pub fn radius_selector(input: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let r2 = radius * radius;
    let mut selected: Vec<usize> = Vec::new();

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        if dx * dx + dy * dy + dz * dz <= r2 {
            selected.push(i);
        }
    }

    extract_points_as_vertices(input, &selected)
}

/// Select cells whose centroid lies within a distance of a line segment.
///
/// The line is defined by two points (p0, p1). `radius` is the maximum
/// distance from the line for a cell to be selected.
pub fn linear_selector(
    input: &PolyData,
    line_start: [f64; 3],
    line_end: [f64; 3],
    radius: f64,
) -> PolyData {
    let dir = [
        line_end[0] - line_start[0],
        line_end[1] - line_start[1],
        line_end[2] - line_start[2],
    ];
    let line_len2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
    let r2 = radius * radius;

    let mut selected: Vec<usize> = Vec::new();

    let num_cells_total = input.polys.num_cells();
    for ci in 0..num_cells_total {
        let cell = input.polys.cell(ci);
        if cell.is_empty() {
            continue;
        }
        // Compute centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &vid in cell {
            let p = input.points.get(vid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;

        // Distance from centroid to line segment
        let to_point = [cx - line_start[0], cy - line_start[1], cz - line_start[2]];
        let t = if line_len2 > 1e-30 {
            ((to_point[0] * dir[0] + to_point[1] * dir[1] + to_point[2] * dir[2]) / line_len2)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };
        let closest = [
            line_start[0] + t * dir[0],
            line_start[1] + t * dir[1],
            line_start[2] + t * dir[2],
        ];
        let dx = cx - closest[0];
        let dy = cy - closest[1];
        let dz = cz - closest[2];
        if dx * dx + dy * dy + dz * dz <= r2 {
            selected.push(ci);
        }
    }

    extract_cells_by_indices(
        input,
        &selected,
        "SelectedByLine",
        &vec![1.0; selected.len()],
    )
}

// --- helpers ---

fn extract_cells_by_indices(
    input: &PolyData,
    cell_indices: &[usize],
    data_name: &str,
    data_values: &[f64],
) -> PolyData {
    // Collect needed points and remap indices
    let num_pts = input.points.len();
    let mut point_map = vec![i64::MAX; num_pts];
    let mut new_points = crate::data::Points::<f64>::new();
    let total_cells = input.polys.num_cells();

    for &ci in cell_indices {
        if ci >= total_cells {
            continue;
        }
        let cell = input.polys.cell(ci);
        for &vid in cell {
            let vi = vid as usize;
            if vi < num_pts && point_map[vi] == i64::MAX {
                point_map[vi] = new_points.len() as i64;
                new_points.push(input.points.get(vi));
            }
        }
    }

    let mut new_polys = crate::data::CellArray::new();
    let mut new_values = Vec::new();
    for (value_idx, &ci) in cell_indices.iter().enumerate() {
        if ci >= total_cells {
            continue;
        }
        let cell = input.polys.cell(ci);
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
        new_polys.push_cell(&remapped);
        new_values.push(data_values.get(value_idx).copied().unwrap_or(0.0));
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            data_name, new_values, 1,
        )));
    result
}

fn extract_points_as_vertices(input: &PolyData, indices: &[usize]) -> PolyData {
    let mut points = crate::data::Points::<f64>::new();
    let mut verts = crate::data::CellArray::new();

    for (new_i, &old_i) in indices.iter().enumerate() {
        points.push(input.points.get(old_i));
        verts.push_cell(&[new_i as i64]);
    }

    let mut result = PolyData::new();
    result.points = points;
    result.verts = verts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_mesh() -> PolyData {
        // 2x2 grid of triangles
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 4], [0, 4, 3], [1, 2, 5], [1, 5, 4]],
        )
    }

    #[test]
    fn cell_distance_from_seed() {
        let mesh = quad_mesh();
        let result = cell_distance_selector(&mesh, &[0], 2);
        assert!(result.polys.num_cells() >= 2);
    }

    #[test]
    fn radius_selection() {
        let mesh = quad_mesh();
        let result = radius_selector(&mesh, [1.0, 0.5, 0.0], 0.6);
        assert!(result.points.len() >= 1);
    }

    #[test]
    fn linear_selection() {
        let mesh = quad_mesh();
        let result = linear_selector(&mesh, [0.0, 0.5, 0.0], [2.0, 0.5, 0.0], 0.6);
        assert!(result.polys.num_cells() >= 2);
    }
}
