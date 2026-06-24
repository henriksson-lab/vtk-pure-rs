use crate::data::{CellArray, Points, PolyData};

/// Rotational extrusion of a 2D profile around the Y axis.
///
/// Takes a PolyData whose line cells represent a 2D profile in the XY plane
/// and sweeps it around the Y axis by `angle` degrees with `resolution` steps.
/// An angle of 360.0 creates a closed surface of revolution.
///
/// The profile should be in the X≥0 half-plane for correct geometry.
pub fn rotation_extrude(input: &PolyData, angle: f64, resolution: usize) -> PolyData {
    let resolution = resolution.max(3);
    let angle_rad = angle.to_radians();
    let closed = (angle - 360.0).abs() < 1e-6;
    let n_steps = if closed { resolution } else { resolution + 1 };

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }

        let profile_len = cell.len();
        let base_offset = points.len();

        // Generate rotated copies of the profile points
        for step in 0..n_steps {
            let theta = angle_rad * (step as f64) / (resolution as f64);
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            for &pid in cell.iter() {
                let p = input.points.get(pid as usize);
                let x = p[0];
                let y = p[1];
                // Rotate around Y axis: (x,y,0) -> (x*cos, y, x*sin)
                points.push([x * cos_t, y, x * sin_t]);
            }
        }

        // Connect consecutive rings with quads
        for step in 0..resolution {
            let ring_a = base_offset + step * profile_len;
            let ring_b = if closed && step == resolution - 1 {
                base_offset // wrap to first ring
            } else {
                base_offset + (step + 1) * profile_len
            };

            for i in 0..profile_len - 1 {
                let a0 = (ring_a + i) as i64;
                let a1 = (ring_a + i + 1) as i64;
                let b0 = (ring_b + i) as i64;
                let b1 = (ring_b + i + 1) as i64;

                // Two triangles per quad
                polys.push_cell(&[a0, a1, b1]);
                polys.push_cell(&[a0, b1, b0]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revolve_line_segment() {
        // A vertical line at x=1 -> should produce a cylinder
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = rotation_extrude(&pd, 360.0, 8);
        // 8 steps, 2 profile points = 16 points (closed, no extra ring)
        assert_eq!(result.points.len(), 16);
        // 8 segments * 1 quad * 2 tris = 16 triangles
        assert_eq!(result.polys.num_cells(), 16);
    }

    #[test]
    fn revolve_partial() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = rotation_extrude(&pd, 180.0, 6);
        // Not closed: 7 rings * 2 = 14 points, 6 segments * 2 tris = 12 tris
        assert_eq!(result.points.len(), 14);
        assert_eq!(result.polys.num_cells(), 12);
    }

    #[test]
    fn revolve_profile() {
        // L-shaped profile
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = rotation_extrude(&pd, 360.0, 4);
        // 4 rings * 3 profile pts = 12 points
        assert_eq!(result.points.len(), 12);
        // 4 segments * 2 quads * 2 tris = 16
        assert_eq!(result.polys.num_cells(), 16);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = rotation_extrude(&pd, 360.0, 8);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
