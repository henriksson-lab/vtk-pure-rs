//! Create a solid of revolution from a 2D profile.
//!
//! Revolves line cells around the Z axis, generating quad surface cells.

use crate::data::PolyData;

/// Revolve line-cell profiles around the Z axis.
///
/// Input: PolyData with lines lying in the XY plane. The points are rotated
/// `num_sides` times around the Z axis (full 360 degrees) and each line segment
/// is swept into a quad.
pub fn volume_of_revolution(input: &PolyData, num_sides: usize) -> PolyData {
    let num_sides = num_sides.max(3);
    let n_points = input.points.len();
    if n_points == 0 {
        return PolyData::new();
    }

    let mut output = PolyData::new();

    // Generate points: for each slice and each input point.
    let angle_step = 2.0 * std::f64::consts::PI / num_sides as f64;
    for si in 0..num_sides {
        let angle = si as f64 * angle_step;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        for pi in 0..n_points {
            let p = input.points.get(pi);
            output.points.push([
                p[0] * cos_a - p[1] * sin_a,
                p[0] * sin_a + p[1] * cos_a,
                p[2],
            ]);
        }
    }

    // Generate quads connecting adjacent slices for all input line segments.
    for si in 0..num_sides {
        let next_si = (si + 1) % num_sides;
        let base = (si * n_points) as i64;
        let next_base = (next_si * n_points) as i64;

        for cell in input.lines.iter() {
            for i in 0..cell.len().saturating_sub(1) {
                let p0 = base + cell[i];
                let p1 = base + cell[i + 1];
                let p2 = next_base + cell[i + 1];
                let p3 = next_base + cell[i];
                output.polys.push_cell(&[p0, p1, p2, p3]);
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revolve_line_segment() {
        // A line segment parallel to Z at radius 1 should produce a cylinder-like shape.
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = volume_of_revolution(&pd, 8);

        // 8 sides * 2 profile points = 16 points
        assert_eq!(result.points.len(), 16);
        // 8 sides * 1 segment = 8 quads
        assert_eq!(result.polys.num_cells(), 8);

        // All points should be at radius ~1.0 from Z axis
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!((r - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn revolves_all_line_cells() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 1.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[2, 3]);

        let result = volume_of_revolution(&pd, 4);

        assert_eq!(result.points.len(), 16);
        assert_eq!(result.polys.num_cells(), 8);
    }
}
