//! Cut a mesh surface by a polygon outline (cookie cutter).
//!
//! Removes cells that fall outside the cutting polygon, producing
//! a mesh trimmed to the polygon boundary.

use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Cut a mesh by a 2D polygon outline, keeping only cells inside.
///
/// The polygon is defined in the XY plane. Convex polygons clip cell
/// boundaries; concave polygons currently fall back to VTK's inside test at
/// the cell centroid.
pub fn cookie_cut(mesh: &PolyData, polygon: &[[f64; 2]]) -> PolyData {
    if polygon.len() < 3 || mesh.polys.num_cells() == 0 {
        return PolyData::new();
    }

    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();
    let mut point_map: HashMap<PointKey, i64> = HashMap::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let cell_points: Vec<[f64; 3]> = cell
            .iter()
            .map(|&pid| mesh.points.get(pid as usize))
            .collect();
        let clipped = if is_convex_polygon(polygon) {
            clip_polygon_to_convex_loop(&cell_points, polygon)
        } else if point_in_polygon_centroid(&cell_points, polygon) {
            cell_points
        } else {
            Vec::new()
        };

        if clipped.len() < 3 {
            continue;
        }

        let mut new_ids = Vec::with_capacity(clipped.len());
        for point in clipped {
            new_ids.push(insert_unique_point(&mut new_points, &mut point_map, point));
        }
        new_polys.push_cell(&new_ids);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

/// Cut by a circular region (keep cells inside circle in XY plane).
pub fn cookie_cut_circle(mesh: &PolyData, center: [f64; 2], radius: f64) -> PolyData {
    if radius <= 0.0 {
        return PolyData::new();
    }

    const NUM_SIDES: usize = 64;
    let polygon: Vec<[f64; 2]> = (0..NUM_SIDES)
        .map(|i| {
            let theta = std::f64::consts::TAU * (i as f64) / (NUM_SIDES as f64);
            [
                center[0] + radius * theta.cos(),
                center[1] + radius * theta.sin(),
            ]
        })
        .collect();
    cookie_cut(mesh, &polygon)
}

fn point_in_polygon(px: f64, py: f64, polygon: &[[f64; 2]]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let yi = polygon[i][1];
        let yj = polygon[j][1];
        if (yi > py) != (yj > py) {
            let x_int = polygon[i][0] + (py - yi) / (yj - yi) * (polygon[j][0] - polygon[i][0]);
            if px < x_int {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

type PointKey = (i64, i64, i64);

fn insert_unique_point(
    points: &mut Points<f64>,
    point_map: &mut HashMap<PointKey, i64>,
    point: [f64; 3],
) -> i64 {
    let key = point_key(point);
    *point_map.entry(key).or_insert_with(|| {
        let idx = points.len() as i64;
        points.push(point);
        idx
    })
}

fn point_key(point: [f64; 3]) -> PointKey {
    const SCALE: f64 = 1.0e9;
    (
        (point[0] * SCALE).round() as i64,
        (point[1] * SCALE).round() as i64,
        (point[2] * SCALE).round() as i64,
    )
}

fn point_in_polygon_centroid(cell_points: &[[f64; 3]], polygon: &[[f64; 2]]) -> bool {
    let mut cx = 0.0;
    let mut cy = 0.0;
    for point in cell_points {
        cx += point[0];
        cy += point[1];
    }
    let n = cell_points.len() as f64;
    point_in_polygon(cx / n, cy / n, polygon)
}

fn is_convex_polygon(polygon: &[[f64; 2]]) -> bool {
    let mut sign = 0.0;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        let c = polygon[(i + 2) % polygon.len()];
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if cross.abs() <= 1.0e-12 {
            continue;
        }
        if sign == 0.0 {
            sign = cross.signum();
        } else if sign * cross < 0.0 {
            return false;
        }
    }
    sign != 0.0
}

fn clip_polygon_to_convex_loop(subject: &[[f64; 3]], clip: &[[f64; 2]]) -> Vec<[f64; 3]> {
    let orientation = signed_area_2d(clip).signum();
    if orientation == 0.0 {
        return Vec::new();
    }

    let mut output = subject.to_vec();
    for i in 0..clip.len() {
        let edge_start = clip[i];
        let edge_end = clip[(i + 1) % clip.len()];
        let input = output;
        output = Vec::new();
        if input.is_empty() {
            break;
        }

        let mut previous = *input.last().unwrap();
        let mut previous_inside = inside_clip_edge(previous, edge_start, edge_end, orientation);
        for &current in &input {
            let current_inside = inside_clip_edge(current, edge_start, edge_end, orientation);
            if current_inside != previous_inside {
                output.push(intersect_segment_with_clip_edge(
                    previous, current, edge_start, edge_end,
                ));
            }
            if current_inside {
                output.push(current);
            }
            previous = current;
            previous_inside = current_inside;
        }
    }

    remove_duplicate_consecutive_points(output)
}

fn signed_area_2d(polygon: &[[f64; 2]]) -> f64 {
    let mut area = 0.0;
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        area += a[0] * b[1] - b[0] * a[1];
    }
    0.5 * area
}

fn inside_clip_edge(
    point: [f64; 3],
    edge_start: [f64; 2],
    edge_end: [f64; 2],
    orientation: f64,
) -> bool {
    let cross = (edge_end[0] - edge_start[0]) * (point[1] - edge_start[1])
        - (edge_end[1] - edge_start[1]) * (point[0] - edge_start[0]);
    orientation * cross >= -1.0e-12
}

fn intersect_segment_with_clip_edge(
    a: [f64; 3],
    b: [f64; 3],
    edge_start: [f64; 2],
    edge_end: [f64; 2],
) -> [f64; 3] {
    let sx = b[0] - a[0];
    let sy = b[1] - a[1];
    let ex = edge_end[0] - edge_start[0];
    let ey = edge_end[1] - edge_start[1];
    let denom = sx * ey - sy * ex;
    if denom.abs() <= 1.0e-15 {
        return b;
    }
    let t = ((edge_start[0] - a[0]) * ey - (edge_start[1] - a[1]) * ex) / denom;
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

fn remove_duplicate_consecutive_points(points: Vec<[f64; 3]>) -> Vec<[f64; 3]> {
    let mut out = Vec::with_capacity(points.len());
    for point in points {
        if out
            .last()
            .map(|&prev| dist2(prev, point) > 1.0e-20)
            .unwrap_or(true)
        {
            out.push(point);
        }
    }
    if out.len() > 1 && dist2(out[0], *out.last().unwrap()) <= 1.0e-20 {
        out.pop();
    }
    out
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    fn make_grid_mesh() -> PolyData {
        // 4x4 grid of triangles in XY plane
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn cut_by_polygon() {
        let mesh = make_grid_mesh();
        let polygon = [[0.5, 0.5], [2.5, 0.5], [2.5, 2.5], [0.5, 2.5]];
        let result = cookie_cut(&mesh, &polygon);
        assert!(result.polys.num_cells() > 0);
        assert!(result.polys.num_cells() < mesh.polys.num_cells());
    }

    #[test]
    fn cut_by_circle() {
        let mesh = make_grid_mesh();
        let result = cookie_cut_circle(&mesh, [2.0, 2.0], 1.5);
        assert!(result.polys.num_cells() > 0);
        assert!(result.polys.num_cells() < mesh.polys.num_cells());
    }

    #[test]
    fn polygon_outside_mesh() {
        let mesh = make_grid_mesh();
        let polygon = [[100.0, 100.0], [200.0, 100.0], [200.0, 200.0]];
        let result = cookie_cut(&mesh, &polygon);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn polygon_covers_all() {
        let mesh = make_grid_mesh();
        let polygon = [[-1.0, -1.0], [10.0, -1.0], [10.0, 10.0], [-1.0, 10.0]];
        let result = cookie_cut(&mesh, &polygon);
        assert_eq!(result.polys.num_cells(), mesh.polys.num_cells());
    }
}
