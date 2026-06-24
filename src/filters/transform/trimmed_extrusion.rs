use crate::data::{CellArray, Points, PolyData};

/// Extrude a 2D mesh along a direction, trimming the extrusion where it
/// would exceed the given axis-aligned bounding box.
///
/// Each point is extruded by `direction * distance`, but the extrusion
/// distance is clamped so that the extruded point stays within `bounds`
/// `[xmin, xmax, ymin, ymax, zmin, zmax]`.
///
/// The output is a PolyData with side-wall triangles connecting original
/// and extruded edges, plus top and bottom caps.
pub fn trimmed_extrusion(
    input: &PolyData,
    direction: [f64; 3],
    max_distance: f64,
    bounds: [f64; 6],
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let dir_len =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    if dir_len < 1e-15 || max_distance <= 0.0 {
        return PolyData::new();
    }

    // Normalize direction
    let dir = [
        direction[0] / dir_len,
        direction[1] / dir_len,
        direction[2] / dir_len,
    ];

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Copy original points
    for i in 0..n {
        out_points.push(input.points.get(i));
    }

    // Compute per-point clamped extrusion and add extruded points
    for i in 0..n {
        let p = input.points.get(i);
        let t = clamp_distance_to_bounds(p, dir, max_distance, bounds);
        out_points.push([p[0] + dir[0] * t, p[1] + dir[1] * t, p[2] + dir[2] * t]);
    }

    let offset = n as i64;

    // Side wall triangles for each polygon edge
    for cell in input.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i];
            let b = cell[(i + 1) % nc];
            // Two triangles per quad: (a, b, b+offset) and (a, b+offset, a+offset)
            out_polys.push_cell(&[a, b, b + offset]);
            out_polys.push_cell(&[a, b + offset, a + offset]);
        }
    }

    // Also handle line cells
    for cell in input.lines.iter() {
        for i in 0..cell.len().saturating_sub(1) {
            let a = cell[i];
            let b = cell[i + 1];
            out_polys.push_cell(&[a, b, b + offset]);
            out_polys.push_cell(&[a, b + offset, a + offset]);
        }
    }

    // Bottom cap (original polygons)
    for cell in input.polys.iter() {
        out_polys.push_cell(cell);
    }

    // Top cap (extruded polygons, reversed winding)
    for cell in input.polys.iter() {
        let reversed: Vec<i64> = cell.iter().rev().map(|&id| id + offset).collect();
        out_polys.push_cell(&reversed);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Compute the maximum extrusion distance for a point such that the
/// extruded position stays within the AABB bounds.
fn clamp_distance_to_bounds(
    point: [f64; 3],
    dir: [f64; 3],
    max_dist: f64,
    bounds: [f64; 6],
) -> f64 {
    let mut t = max_dist;

    // For each axis, clamp t so point + dir*t stays within [min, max]
    for axis in 0..3 {
        let p = point[axis];
        let d = dir[axis];
        let lo = bounds[axis * 2];
        let hi = bounds[axis * 2 + 1];

        if d.abs() < 1e-15 {
            continue;
        }

        // p + d*t <= hi  =>  t <= (hi - p) / d  (if d > 0)
        // p + d*t >= lo  =>  t <= (lo - p) / d  (if d < 0)
        if d > 0.0 {
            let t_max = (hi - p) / d;
            if t_max < t {
                t = t_max;
            }
        } else {
            let t_max = (lo - p) / d;
            if t_max < t {
                t = t_max;
            }
        }
    }

    if t < 0.0 {
        t = 0.0;
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trimmed_extrusion_basic() {
        // Triangle on XY plane at z=0
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // Extrude along +Z with max distance 10, but bounds limit to z=2
        let result = trimmed_extrusion(
            &pd,
            [0.0, 0.0, 1.0],
            10.0,
            [-10.0, 10.0, -10.0, 10.0, -1.0, 2.0],
        );

        assert_eq!(result.points.len(), 6); // 3 original + 3 extruded

        // Check that extruded points are at z=2 (clamped from 10)
        for i in 3..6 {
            let p = result.points.get(i);
            assert!((p[2] - 2.0).abs() < 1e-10, "expected z=2.0, got z={}", p[2]);
        }

        // Side wall triangles: 3 edges * 2 tris = 6, plus 1 bottom + 1 top cap = 8
        assert_eq!(result.polys.num_cells(), 8);
    }
}
