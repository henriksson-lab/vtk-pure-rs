use crate::data::{CellArray, Points, PolyData};
use crate::types::ImplicitFunction;

/// Clip a PolyData mesh with an implicit function.
///
/// Removes cells where all vertices have `f(point) > 0` (outside).
/// Keeps cells where at least one vertex has `f(point) <= 0` (inside).
pub fn clip_with_implicit(input: &PolyData, func: &dyn ImplicitFunction) -> PolyData {
    let n = input.points.len();
    let values: Vec<f64> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            func.evaluate(p[0], p[1], p[2])
        })
        .collect();

    let mut used = vec![false; n];
    let mut kept_cells = Vec::new();

    for (ci, cell) in input.polys.iter().enumerate() {
        // Keep cell if any vertex is inside (value <= 0)
        let any_inside = cell.iter().any(|&pid| values[pid as usize] <= 0.0);
        if any_inside {
            kept_cells.push(ci);
            for &pid in cell {
                used[pid as usize] = true;
            }
        }
    }

    // Remap points
    let mut old_to_new = vec![usize::MAX; n];
    let mut new_points = Points::new();
    for (i, &u) in used.iter().enumerate() {
        if u {
            old_to_new[i] = new_points.len();
            new_points.push(input.points.get(i));
        }
    }

    let mut new_polys = CellArray::new();
    for &ci in &kept_cells {
        let cell = input.polys.cell(ci);
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&pid| old_to_new[pid as usize] as i64)
            .collect();
        new_polys.push_cell(&remapped);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
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
        // Plane at x=1.5, normal pointing +X → keeps x < 1.5
        let plane = ImplicitPlane::new([1.5, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let clipped = clip_with_implicit(&pd, &plane);
        // First triangle has vertices at x=-1,1,0 → all inside
        // Second triangle has vertices at x=1,2,2 → vertex at x=2 is outside, but x=1 is inside
        assert_eq!(clipped.polys.num_cells(), 2); // both kept (each has at least one inside vertex)
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
        assert_eq!(clipped.polys.num_cells(), 1); // only inside triangle kept
    }

    #[test]
    fn clip_keeps_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 10.0);
        let clipped = clip_with_implicit(&pd, &sphere);
        assert_eq!(clipped.polys.num_cells(), 1);
    }
}
