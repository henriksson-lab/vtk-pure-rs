use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Select faces whose normal is within a given angle of a reference direction.
///
/// Returns a new PolyData containing only the faces whose outward normal
/// makes an angle less than or equal to `max_angle_deg` with `direction`.
/// Points are compacted so only referenced vertices are included.
pub fn select_faces_by_normal(
    input: &PolyData,
    direction: [f64; 3],
    max_angle_deg: f64,
) -> PolyData {
    let dir_len: f64 =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dir_len < 1e-15 {
        return PolyData::new();
    }
    let dx: f64 = direction[0] / dir_len;
    let dy: f64 = direction[1] / dir_len;
    let dz: f64 = direction[2] / dir_len;
    let cos_threshold: f64 = max_angle_deg.to_radians().cos();

    let mut new_points = Points::new();
    let mut new_polys = CellArray::new();
    let mut point_map: HashMap<usize, i64> = HashMap::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Compute face normal using first 3 vertices
        let p0 = input.points.get(cell[0] as usize);
        let p1 = input.points.get(cell[1] as usize);
        let p2 = input.points.get(cell[2] as usize);

        let e1: [f64; 3] = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2: [f64; 3] = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];

        let nx: f64 = e1[1] * e2[2] - e1[2] * e2[1];
        let ny: f64 = e1[2] * e2[0] - e1[0] * e2[2];
        let nz: f64 = e1[0] * e2[1] - e1[1] * e2[0];
        let nlen: f64 = (nx * nx + ny * ny + nz * nz).sqrt();

        if nlen < 1e-15 {
            continue;
        }

        let cos_angle: f64 = (nx * dx + ny * dy + nz * dz) / nlen;

        if cos_angle >= cos_threshold {
            // Remap point indices
            let new_cell: Vec<i64> = cell
                .iter()
                .map(|&old_id| {
                    let old: usize = old_id as usize;
                    *point_map.entry(old).or_insert_with(|| {
                        let idx: i64 = new_points.len() as i64;
                        new_points.push(input.points.get(old));
                        idx
                    })
                })
                .collect();
            new_polys.push_cell(&new_cell);
        }
    }

    let mut pd = PolyData::new();
    pd.points = new_points;
    pd.polys = new_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_upward_faces() {
        // Two triangles: one facing up (+z), one facing down (-z)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0], // +z normal
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 0.0, 1.0], // -z normal (reversed winding)
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 45.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn wide_angle_selects_all() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 180.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn zero_angle_exact_match() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Normal is exactly +z, selecting with direction +z and angle 0
        let result = select_faces_by_normal(&pd, [0.0, 0.0, 1.0], 0.0);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
