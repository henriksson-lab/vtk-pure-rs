use crate::data::{CellArray, Points, PolyData};

/// Clip a mesh by a cylinder, keeping points inside or outside.
///
/// The cylinder is defined by a `center` point, an `axis` direction, and a `radius`.
/// Points are classified as inside if their squared perpendicular distance to
/// the axis is less than or equal to the squared radius. If `keep_inside` is true, cells whose **all** vertices
/// are inside the cylinder are kept; otherwise cells whose all vertices are outside.
pub fn clip_by_cylinder(
    input: &PolyData,
    center: [f64; 3],
    axis: [f64; 3],
    radius: f64,
    keep_inside: bool,
) -> PolyData {
    // Normalize the axis direction.
    let axis_len: f64 = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if axis_len < 1e-15 {
        return PolyData::new();
    }
    let ax: [f64; 3] = [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len];
    let radius_sq = radius * radius;

    // Classify each point: true means inside the cylinder.
    let n: usize = input.points.len();
    let mut inside = vec![false; n];
    for i in 0..n {
        let p = input.points.get(i);
        let dx: f64 = p[0] - center[0];
        let dy: f64 = p[1] - center[1];
        let dz: f64 = p[2] - center[2];
        let proj: f64 = dx * ax[0] + dy * ax[1] + dz * ax[2];
        let perp_sq: f64 = dx * dx + dy * dy + dz * dz - proj * proj;
        inside[i] = perp_sq <= radius_sq;
    }

    // Build output: keep cells whose all vertices satisfy the condition.
    let mut new_points = Points::new();
    let mut new_polys = CellArray::new();
    let mut point_map: Vec<Option<i64>> = vec![None; n];
    let mut next_id: i64 = 0;

    for cell in input.polys.iter() {
        let all_match = cell.iter().all(|&id| {
            let flag = inside[id as usize];
            if keep_inside {
                flag
            } else {
                !flag
            }
        });
        if !all_match {
            continue;
        }
        let mut new_cell = Vec::with_capacity(cell.len());
        for &id in cell {
            let idx = id as usize;
            if point_map[idx].is_none() {
                new_points.push(input.points.get(idx));
                point_map[idx] = Some(next_id);
                next_id += 1;
            }
            new_cell.push(point_map[idx].unwrap());
        }
        new_polys.push_cell(&new_cell);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> PolyData {
        // Two triangles: one at origin, one at x=10
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        )
    }

    #[test]
    fn keep_inside_cylinder() {
        let mesh = sample_mesh();
        // Cylinder centered at origin with Z axis, radius 5 -> first triangle inside
        let result = clip_by_cylinder(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0, true);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn keep_outside_cylinder() {
        let mesh = sample_mesh();
        // Same cylinder, keep outside -> second triangle
        let result = clip_by_cylinder(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0, false);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn large_radius_keeps_all() {
        let mesh = sample_mesh();
        let result = clip_by_cylinder(&mesh, [5.0, 0.0, 0.0], [0.0, 0.0, 1.0], 100.0, true);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 6);
    }
}
