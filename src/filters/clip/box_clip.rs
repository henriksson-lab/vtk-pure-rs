use crate::data::{CellArray, Points, PolyData};

/// Clip PolyData by an axis-aligned bounding box.
///
/// `bounds` is `[xmin, xmax, ymin, ymax, zmin, zmax]`.
pub fn box_clip(input: &PolyData, bounds: [f64; 6]) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut out_lines = CellArray::new();
    let mut out_polys = CellArray::new();

    for cell in input.verts.iter() {
        for &pid in cell {
            let p = input.points.get(pid as usize);
            if point_in_bounds(p, bounds) {
                let id = out_points.len() as i64;
                out_points.push(p);
                out_verts.push_cell(&[id]);
            }
        }
    }

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() - 1 {
            let p0 = input.points.get(cell[i] as usize);
            let p1 = input.points.get(cell[i + 1] as usize);
            if let Some((c0, c1)) = clip_line_to_box(p0, p1, bounds) {
                let id0 = out_points.len() as i64;
                out_points.push(c0);
                let id1 = out_points.len() as i64;
                out_points.push(c1);
                out_lines.push_cell(&[id0, id1]);
            }
        }
    }

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let mut polygon: Vec<[f64; 3]> = cell
            .iter()
            .map(|&pid| input.points.get(pid as usize))
            .collect();
        for plane in 0..6 {
            polygon = clip_polygon_to_box_plane(&polygon, bounds, plane);
            if polygon.len() < 3 {
                break;
            }
        }
        if polygon.len() < 3 {
            continue;
        }

        let base = out_points.len() as i64;
        for p in &polygon {
            out_points.push(*p);
        }
        for i in 1..polygon.len() - 1 {
            out_polys.push_cell(&[base, base + i as i64, base + (i + 1) as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.verts = out_verts;
    result.lines = out_lines;
    result.polys = out_polys;
    result
}

fn point_in_bounds(p: [f64; 3], bounds: [f64; 6]) -> bool {
    p[0] >= bounds[0]
        && p[0] <= bounds[1]
        && p[1] >= bounds[2]
        && p[1] <= bounds[3]
        && p[2] >= bounds[4]
        && p[2] <= bounds[5]
}

fn clip_line_to_box(p0: [f64; 3], p1: [f64; 3], bounds: [f64; 6]) -> Option<([f64; 3], [f64; 3])> {
    let mut t0 = 0.0;
    let mut t1 = 1.0;
    let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];

    for axis in 0..3 {
        let min_bound = bounds[2 * axis];
        let max_bound = bounds[2 * axis + 1];
        if d[axis].abs() <= 1e-15 {
            if p0[axis] < min_bound || p0[axis] > max_bound {
                return None;
            }
            continue;
        }

        let inv_d = 1.0 / d[axis];
        let mut t_near = (min_bound - p0[axis]) * inv_d;
        let mut t_far = (max_bound - p0[axis]) * inv_d;
        if t_near > t_far {
            std::mem::swap(&mut t_near, &mut t_far);
        }
        if t_near > t0 {
            t0 = t_near;
        }
        if t_far < t1 {
            t1 = t_far;
        }
        if t0 > t1 {
            return None;
        }
    }

    Some((lerp3(p0, p1, t0), lerp3(p0, p1, t1)))
}

fn clip_polygon_to_box_plane(
    polygon: &[[f64; 3]],
    bounds: [f64; 6],
    plane: usize,
) -> Vec<[f64; 3]> {
    if polygon.is_empty() {
        return Vec::new();
    }

    let inside = |p: [f64; 3]| -> bool {
        match plane {
            0 => p[0] >= bounds[0],
            1 => p[0] <= bounds[1],
            2 => p[1] >= bounds[2],
            3 => p[1] <= bounds[3],
            4 => p[2] >= bounds[4],
            5 => p[2] <= bounds[5],
            _ => unreachable!(),
        }
    };
    let value = |p: [f64; 3]| -> f64 {
        match plane {
            0 | 1 => p[0],
            2 | 3 => p[1],
            4 | 5 => p[2],
            _ => unreachable!(),
        }
    };
    let limit = bounds[plane];

    let mut out = Vec::new();
    for i in 0..polygon.len() {
        let j = (i + 1) % polygon.len();
        let pi = polygon[i];
        let pj = polygon[j];
        let i_in = inside(pi);
        let j_in = inside(pj);

        if i_in {
            out.push(pi);
        }

        if i_in != j_in {
            let vi = value(pi);
            let vj = value(pj);
            let denom = vj - vi;
            if denom.abs() > 1e-15 {
                let t = ((limit - vi) / denom).clamp(0.0, 1.0);
                out.push([
                    pi[0] + t * (pj[0] - pi[0]),
                    pi[1] + t * (pj[1] - pi[1]),
                    pi[2] + t * (pj[2] - pi[2]),
                ]);
            }
        }
    }

    out
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_keeps_inside() {
        let pd = PolyData::from_triangles(
            vec![
                [0.5, 0.5, 0.0],
                [1.0, 0.5, 0.0],
                [0.75, 1.0, 0.0],
                // Outside triangle
                [5.0, 5.0, 0.0],
                [6.0, 5.0, 0.0],
                [5.5, 6.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let result = box_clip(&pd, [0.0, 2.0, 0.0, 2.0, -1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn clip_splits_intersecting_polygon() {
        let pd = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [0.0, 2.0, -1.0, 2.0, -1.0, 1.0]);
        assert!(result.polys.num_cells() > 0);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }

    #[test]
    fn clip_removes_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [10.0, 20.0, 10.0, 20.0, 10.0, 20.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn clip_keeps_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [-10.0, 10.0, -10.0, 10.0, -10.0, 10.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
