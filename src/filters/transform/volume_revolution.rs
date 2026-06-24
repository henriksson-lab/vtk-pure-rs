//! Create a solid of revolution from a 2D profile.
//!
//! Revolves a polyline in the XY plane around the Y axis, generating a
//! triangle mesh surface.

use crate::data::PolyData;

/// Revolve a 2D polyline profile around the Y axis.
///
/// Input: PolyData with a polyline (in `lines`) lying in the XY plane.
/// The profile is rotated `num_sides` times around the Y axis (full 360
/// degrees) to create a triangle mesh.
///
/// Points are extracted in order from the first line cell.
pub fn volume_of_revolution(input: &PolyData, num_sides: usize) -> PolyData {
    let num_sides = num_sides.max(3);

    // Extract profile points from the first line cell, or fall back to all points in order
    let profile: Vec<[f64; 2]> = if input.lines.num_cells() > 0 {
        let cell = input.lines.cell(0);
        cell.iter()
            .map(|&id| {
                let p = input.points.get(id as usize);
                [p[0], p[1]]
            })
            .collect()
    } else {
        (0..input.points.len())
            .map(|i| {
                let p = input.points.get(i);
                [p[0], p[1]]
            })
            .collect()
    };

    let n_profile = profile.len();
    if n_profile < 2 {
        return PolyData::new();
    }

    let mut output = PolyData::new();

    // Generate points: for each slice and each profile point
    let angle_step = 2.0 * std::f64::consts::PI / num_sides as f64;
    for si in 0..num_sides {
        let angle = si as f64 * angle_step;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        for pp in &profile {
            let x = pp[0]; // radius from Y axis
            let y = pp[1]; // height along Y axis
                           // Rotate (x, 0) around Y axis
            output.points.push([x * cos_a, y, x * sin_a]);
        }
    }

    // Generate triangles connecting adjacent slices
    for si in 0..num_sides {
        let next_si = (si + 1) % num_sides;
        let base = (si * n_profile) as i64;
        let next_base = (next_si * n_profile) as i64;

        for pi in 0..(n_profile - 1) {
            let p0 = base + pi as i64;
            let p1 = base + (pi + 1) as i64;
            let p2 = next_base + pi as i64;
            let p3 = next_base + (pi + 1) as i64;

            // Two triangles per quad
            output.polys.push_cell(&[p0, p1, p3]);
            output.polys.push_cell(&[p0, p3, p2]);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revolve_line_segment() {
        // A vertical line segment at x=1: should produce a cylinder-like shape
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = volume_of_revolution(&pd, 8);

        // 8 sides * 2 profile points = 16 points
        assert_eq!(result.points.len(), 16);
        // 8 sides * 1 segment * 2 triangles = 16 triangles
        assert_eq!(result.polys.num_cells(), 16);

        // All points should be at radius ~1.0 from Y axis
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            let r = (p[0] * p[0] + p[2] * p[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-10);
        }
    }
}
