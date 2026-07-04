use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

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

    // Classify each point: true means inside the cylinder.
    let n: usize = input.points.len();
    let mut inside = vec![false; n];
    let radius_sq = radius * radius;
    for i in 0..n {
        let p = input.points.get(i);
        let dx: f64 = p[0] - center[0];
        let dy: f64 = p[1] - center[1];
        let dz: f64 = p[2] - center[2];
        let proj: f64 = dx * ax[0] + dy * ax[1] + dz * ax[2];
        let perp_sq: f64 = dx * dx + dy * dy + dz * dz - proj * proj;
        inside[i] = perp_sq.max(0.0) <= radius_sq;
    }

    // Build output: keep cells whose all vertices satisfy the condition.
    let mut new_points = Points::new();
    let mut new_polys = CellArray::new();
    let mut point_map: Vec<Option<i64>> = vec![None; n];
    let mut point_ids = Vec::new();
    let mut cell_ids = Vec::new();
    let mut next_id: i64 = 0;

    for (cell_id, cell) in input.polys.iter().enumerate() {
        let all_match = cell.iter().all(|&id| {
            let Some(idx) = valid_point_id(id, n) else {
                return false;
            };
            let flag = inside[idx];
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
            let idx = valid_point_id(id, n).expect("cell ids were validated above");
            if point_map[idx].is_none() {
                new_points.push(input.points.get(idx));
                point_map[idx] = Some(next_id);
                point_ids.push(idx);
                next_id += 1;
            }
            new_cell.push(point_map[idx].unwrap());
        }
        new_polys.push_cell(&new_cell);
        cell_ids.push(cell_id);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    copy_attribute_tuples(input.point_data(), result.point_data_mut(), &point_ids);
    copy_attribute_tuples(input.cell_data(), result.cell_data_mut(), &cell_ids);
    result
}

fn copy_attribute_tuples(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if tuple_ids
            .iter()
            .all(|&tuple_id| tuple_id < array.num_tuples())
        {
            target.add_array(subset_array(array, tuple_ids));
        }
    }

    if let Some(array) = source.scalars() {
        target.set_active_scalars(array.name());
    }
    if let Some(array) = source.vectors() {
        target.set_active_vectors(array.name());
    }
    if let Some(array) = source.normals() {
        target.set_active_normals(array.name());
    }
    if let Some(array) = source.tcoords() {
        target.set_active_tcoords(array.name());
    }
    if let Some(array) = source.tensors() {
        target.set_active_tensors(array.name());
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! subset {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                data.extend_from_slice($arr.tuple(tuple_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($arr.name(), data, nc))
        }};
    }

    match array {
        AnyDataArray::F32(arr) => subset!(arr, F32),
        AnyDataArray::F64(arr) => subset!(arr, F64),
        AnyDataArray::I8(arr) => subset!(arr, I8),
        AnyDataArray::I16(arr) => subset!(arr, I16),
        AnyDataArray::I32(arr) => subset!(arr, I32),
        AnyDataArray::I64(arr) => subset!(arr, I64),
        AnyDataArray::U8(arr) => subset!(arr, U8),
        AnyDataArray::U16(arr) => subset!(arr, U16),
        AnyDataArray::U32(arr) => subset!(arr, U32),
        AnyDataArray::U64(arr) => subset!(arr, U64),
    }
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
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

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut mesh = sample_mesh();
        mesh.polys.push_cell(&[0, -1, 2]);
        mesh.polys.push_cell(&[0, 99, 2]);

        let result = clip_by_cylinder(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0, true);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn keeps_boundary_points_inside() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = clip_by_cylinder(&mesh, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0, true);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
