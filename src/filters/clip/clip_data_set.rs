use std::collections::HashMap;

use crate::data::{Points, UnstructuredGrid};
use crate::types::CellType;

/// Clip an UnstructuredGrid by a plane defined by a point and normal.
///
/// Keeps cells in the half-space where `dot(p - origin, normal) >= 0`.
/// Linear line and surface cells that cross the plane are split.
pub fn clip_data_set(
    input: &UnstructuredGrid,
    origin: [f64; 3],
    normal: [f64; 3],
) -> UnstructuredGrid {
    let n_points = input.points.len();

    // Classify each point
    let dists: Vec<f64> = (0..n_points)
        .map(|i| {
            let p = input.points.get(i);
            (p[0] - origin[0]) * normal[0]
                + (p[1] - origin[1]) * normal[1]
                + (p[2] - origin[2]) * normal[2]
        })
        .collect();

    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut edge_locator: HashMap<(i64, i64), i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out = UnstructuredGrid::new();

    let n_cells = input.cells().num_cells();
    for ci in 0..n_cells {
        let pts = input.cell_points(ci);
        let ct = input.cell_type(ci);

        // Keep intact cells directly, like vtkClipDataSet does before invoking cell clipping.
        let all_inside = pts.iter().all(|&id| dists[id as usize] >= 0.0);
        if all_inside {
            let remapped =
                remap_existing_points(pts, &mut point_map, &input.points, &mut out_points);
            out.push_cell(ct, &remapped);
            continue;
        }

        let any_inside = pts.iter().any(|&id| dists[id as usize] >= 0.0);
        if !any_inside {
            continue;
        }

        match ct {
            CellType::Line => {
                let clipped = clip_line_cell(
                    pts,
                    &dists,
                    &mut point_map,
                    &mut edge_locator,
                    input,
                    &mut out_points,
                );
                if clipped.len() == 2 {
                    out.push_cell(CellType::Line, &clipped);
                }
            }
            CellType::Triangle | CellType::Quad | CellType::Polygon => {
                let clipped = clip_linear_cell(
                    pts,
                    &dists,
                    &mut point_map,
                    &mut edge_locator,
                    input,
                    &mut out_points,
                );
                if clipped.len() >= 3 {
                    for i in 1..clipped.len() - 1 {
                        out.push_cell(
                            CellType::Triangle,
                            &[clipped[0], clipped[i], clipped[i + 1]],
                        );
                    }
                }
            }
            _ => {}
        }
    }

    out.points = out_points;
    out
}

fn remap_existing_points(
    ids: &[i64],
    point_map: &mut HashMap<usize, usize>,
    input_points: &Points<f64>,
    out_points: &mut Points<f64>,
) -> Vec<i64> {
    ids.iter()
        .map(|&id| {
            let uid = id as usize;
            *point_map.entry(uid).or_insert_with(|| {
                let idx = out_points.len();
                out_points.push(input_points.get(uid));
                idx
            }) as i64
        })
        .collect()
}

fn get_or_insert_intersection(
    a: i64,
    b: i64,
    da: f64,
    db: f64,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
) -> i64 {
    let edge_key = if a < b { (a, b) } else { (b, a) };
    if let Some(&id) = edge_locator.get(&edge_key) {
        return id;
    }

    let t = da / (da - db);
    let pa = input.points.get(a as usize);
    let pb = input.points.get(b as usize);
    let p = [
        pa[0] + t * (pb[0] - pa[0]),
        pa[1] + t * (pb[1] - pa[1]),
        pa[2] + t * (pb[2] - pa[2]),
    ];
    let id = out_points.len() as i64;
    out_points.push(p);
    edge_locator.insert(edge_key, id);
    id
}

fn clip_linear_cell(
    ids: &[i64],
    dists: &[f64],
    point_map: &mut HashMap<usize, usize>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
) -> Vec<i64> {
    let mut result = Vec::new();
    for i in 0..ids.len() {
        let j = if ids.len() == 2 {
            1 - i
        } else {
            (i + 1) % ids.len()
        };
        if ids.len() == 2 && i == 1 {
            break;
        }
        let di = dists[ids[i] as usize];
        let dj = dists[ids[j] as usize];

        if di >= 0.0 {
            let mapped = remap_existing_points(&[ids[i]], point_map, &input.points, out_points);
            result.push(mapped[0]);
        }

        if (di >= 0.0) != (dj >= 0.0) {
            result.push(get_or_insert_intersection(
                ids[i],
                ids[j],
                di,
                dj,
                input,
                out_points,
                edge_locator,
            ));
        }
    }
    result
}

fn clip_line_cell(
    ids: &[i64],
    dists: &[f64],
    point_map: &mut HashMap<usize, usize>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
) -> Vec<i64> {
    if ids.len() != 2 {
        return Vec::new();
    }

    let d0 = dists[ids[0] as usize];
    let d1 = dists[ids[1] as usize];
    let in0 = d0 >= 0.0;
    let in1 = d1 >= 0.0;
    if in0 && in1 {
        return remap_existing_points(ids, point_map, &input.points, out_points);
    }
    if !in0 && !in1 {
        return Vec::new();
    }

    let x = get_or_insert_intersection(ids[0], ids[1], d0, d1, input, out_points, edge_locator);
    if in0 {
        let p0 = remap_existing_points(&[ids[0]], point_map, &input.points, out_points)[0];
        vec![p0, x]
    } else {
        let p1 = remap_existing_points(&[ids[1]], point_map, &input.points, out_points)[0];
        vec![x, p1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CellType;

    #[test]
    fn clip_keeps_inside_cells() {
        let mut grid = UnstructuredGrid::new();
        // Two triangles: one at x>0, one at x<0
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([2.0, 0.0, 0.0]);
        grid.points.push([1.5, 1.0, 0.0]);
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([-2.0, 0.0, 0.0]);
        grid.points.push([-1.5, 1.0, 0.0]);

        grid.push_cell(CellType::Triangle, &[0, 1, 2]);
        grid.push_cell(CellType::Triangle, &[3, 4, 5]);

        // Clip at x=0, keep x>0
        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn clip_removes_all() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([-2.0, 0.0, 0.0]);
        grid.points.push([-1.5, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 0);
    }

    #[test]
    fn clip_mixed_cells() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        // Clip at x=0.5, normal=[1,0,0] — vertex 0 is at boundary (dist=0.5), but
        // all vertices have x >= 0, so the tetra should be kept
        let result = clip_data_set(&grid, [-0.1, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
    }

    #[test]
    fn clip_splits_triangle_cell() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(result.cells().num_cells() > 0);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }
}
