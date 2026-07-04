use crate::data::{CellArray, Points, PolyData};

/// Clip a mesh by a sphere, keeping points inside or outside.
///
/// Points are classified as inside if their distance to `center` is less than `radius`.
/// If `keep_inside` is true, cells whose **all** vertices are inside the sphere are kept;
/// otherwise cells whose all vertices are outside.
pub fn clip_by_sphere(
    input: &PolyData,
    center: [f64; 3],
    radius: f64,
    keep_inside: bool,
) -> PolyData {
    let r_sq: f64 = radius * radius;

    // Classify each point.
    let n: usize = input.points.len();
    let mut inside = vec![false; n];
    for i in 0..n {
        let p = input.points.get(i);
        let dx: f64 = p[0] - center[0];
        let dy: f64 = p[1] - center[1];
        let dz: f64 = p[2] - center[2];
        let dist_sq: f64 = dx * dx + dy * dy + dz * dz;
        inside[i] = dist_sq < r_sq;
    }

    // Build output keeping matching cells.
    let mut new_points = Points::new();
    let mut point_map: Vec<Option<i64>> = vec![None; n];
    let mut next_id: i64 = 0;

    let new_verts = clip_cells(
        &input.verts,
        input,
        &inside,
        keep_inside,
        &mut new_points,
        &mut point_map,
        &mut next_id,
    );
    let new_lines = clip_cells(
        &input.lines,
        input,
        &inside,
        keep_inside,
        &mut new_points,
        &mut point_map,
        &mut next_id,
    );
    let new_polys = clip_cells(
        &input.polys,
        input,
        &inside,
        keep_inside,
        &mut new_points,
        &mut point_map,
        &mut next_id,
    );
    let new_strips = clip_cells(
        &input.strips,
        input,
        &inside,
        keep_inside,
        &mut new_points,
        &mut point_map,
        &mut next_id,
    );

    let mut result = PolyData::new();
    result.points = new_points;
    result.verts = new_verts;
    result.lines = new_lines;
    result.polys = new_polys;
    result.strips = new_strips;
    result
}

fn clip_cells(
    cells: &CellArray,
    input: &PolyData,
    inside: &[bool],
    keep_inside: bool,
    new_points: &mut Points<f64>,
    point_map: &mut [Option<i64>],
    next_id: &mut i64,
) -> CellArray {
    let n = input.points.len();
    let mut new_cells = CellArray::new();
    for cell in cells.iter() {
        let Some(valid_cell) = valid_cell_point_ids(cell, n) else {
            continue;
        };
        let all_match = valid_cell.iter().all(|&idx| {
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
        let mut new_cell = Vec::with_capacity(valid_cell.len());
        for &idx in &valid_cell {
            if point_map[idx].is_none() {
                new_points.push(input.points.get(idx));
                point_map[idx] = Some(*next_id);
                *next_id += 1;
            }
            new_cell.push(point_map[idx].unwrap());
        }
        new_cells.push_cell(&new_cell);
    }
    new_cells
}

fn valid_cell_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    let mut ids = Vec::with_capacity(cell.len());
    for &point_id in cell {
        ids.push(
            usize::try_from(point_id)
                .ok()
                .filter(|&idx| idx < n_points)?,
        );
    }
    Some(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> PolyData {
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [20.0, 0.0, 0.0],
                [21.0, 0.0, 0.0],
                [20.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        )
    }

    #[test]
    fn keep_inside_sphere() {
        let mesh = sample_mesh();
        let result = clip_by_sphere(&mesh, [0.0, 0.0, 0.0], 5.0, true);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn keep_outside_sphere() {
        let mesh = sample_mesh();
        let result = clip_by_sphere(&mesh, [0.0, 0.0, 0.0], 5.0, false);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn huge_sphere_keeps_all_inside() {
        let mesh = sample_mesh();
        let result = clip_by_sphere(&mesh, [10.0, 0.0, 0.0], 100.0, true);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn skips_invalid_cells() {
        let mut mesh = sample_mesh();
        mesh.polys.push_cell(&[0, 1, 99]);
        mesh.polys.push_cell(&[0, -1, 2]);

        let result = clip_by_sphere(&mesh, [0.0, 0.0, 0.0], 5.0, true);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn preserves_all_cell_arrays() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([0.0, 0.0, 1.0]);
        mesh.points.push([10.0, 0.0, 0.0]);
        mesh.points.push([11.0, 0.0, 0.0]);

        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);
        mesh.lines.push_cell(&[4, 5]);

        let result = clip_by_sphere(&mesh, [0.0, 0.0, 0.0], 5.0, true);
        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.points.len(), 4);
    }
}
