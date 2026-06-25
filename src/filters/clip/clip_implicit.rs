use crate::data::{CellArray, Points, PolyData};
use crate::types::ImplicitFunction;

/// Clip a PolyData mesh with an implicit function.
///
/// Keeps the region where `f(point) > 0`, matching vtkClipPolyData's
/// default InsideOut=off sense for Value=0. Polygons crossing the implicit
/// boundary are split by linear interpolation.
pub fn clip_with_implicit(input: &PolyData, func: &dyn ImplicitFunction) -> PolyData {
    let n = input.points.len();
    let values: Vec<f64> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            func.evaluate(p[0], p[1], p[2])
        })
        .collect();

    let mut points = input.points.clone();
    let mut polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        for ti in 1..cell.len() - 1 {
            let ids = [cell[0], cell[ti], cell[ti + 1]];
            let scalars = [
                values[ids[0] as usize],
                values[ids[1] as usize],
                values[ids[2] as usize],
            ];
            let all_inside = scalars.iter().all(|&s| s > 0.0);
            let all_outside = scalars.iter().all(|&s| s <= 0.0);

            if all_inside {
                polys.push_cell(&ids);
            } else if !all_outside {
                let clipped = clip_triangle(&ids, &scalars, &input.points, &mut points);
                if clipped.len() >= 3 {
                    for i in 1..clipped.len() - 1 {
                        polys.push_cell(&[clipped[0], clipped[i], clipped[i + 1]]);
                    }
                }
            }
        }
    }

    compact_poly_data(points, polys)
}

fn clip_triangle(
    ids: &[i64; 3],
    scalars: &[f64; 3],
    input_points: &Points<f64>,
    points: &mut Points<f64>,
) -> Vec<i64> {
    let mut result = Vec::new();

    for i in 0..3 {
        let j = (i + 1) % 3;
        let si = scalars[i];
        let sj = scalars[j];

        if si > 0.0 {
            result.push(ids[i]);
        }

        if (si > 0.0) != (sj > 0.0) {
            let ds = sj - si;
            if ds.abs() > 1e-15 {
                let t = (-si / ds).clamp(0.0, 1.0);
                let pi = input_points.get(ids[i] as usize);
                let pj = input_points.get(ids[j] as usize);
                let p = [
                    pi[0] + t * (pj[0] - pi[0]),
                    pi[1] + t * (pj[1] - pi[1]),
                    pi[2] + t * (pj[2] - pi[2]),
                ];
                let id = points.len() as i64;
                points.push(p);
                result.push(id);
            }
        }
    }

    result
}

fn compact_poly_data(points: Points<f64>, polys: CellArray) -> PolyData {
    let mut used = vec![false; points.len()];
    for cell in polys.iter() {
        for &id in cell {
            used[id as usize] = true;
        }
    }

    let mut point_map = vec![0i64; points.len()];
    let mut compact_points = Points::new();
    for (old_id, is_used) in used.into_iter().enumerate() {
        if is_used {
            point_map[old_id] = compact_points.len() as i64;
            compact_points.push(points.get(old_id));
        }
    }

    let mut compact_polys = CellArray::new();
    for cell in polys.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| point_map[id as usize]).collect();
        compact_polys.push_cell(&remapped);
    }

    let mut result = PolyData::new();
    result.points = compact_points;
    result.polys = compact_polys;
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
        // Plane at x=1.5, normal pointing +X; vtkClipPolyData's default
        // InsideOut=off keeps f > 0, i.e. x > 1.5.
        let plane = ImplicitPlane::new([1.5, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let clipped = clip_with_implicit(&pd, &plane);
        assert_eq!(clipped.polys.num_cells(), 2);
        for i in 0..clipped.points.len() {
            assert!(clipped.points.get(i)[0] >= 1.5 - 1e-10);
        }
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
        assert_eq!(clipped.polys.num_cells(), 1);
        for i in 0..clipped.points.len() {
            let p = clipped.points.get(i);
            assert!(p[0] * p[0] + p[1] * p[1] + p[2] * p[2] >= 1.0 - 1e-10);
        }
    }

    #[test]
    fn clip_keeps_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let sphere = ImplicitSphere::new([0.0, 0.0, 0.0], 10.0);
        let clipped = clip_with_implicit(&pd, &sphere);
        assert_eq!(clipped.polys.num_cells(), 0);
    }
}
