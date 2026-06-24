//! Extract fiber surfaces from bivariate scalar fields.
//!
//! Given a PolyData with two scalar arrays (f, g) and a fiber curve in range
//! space, extract the preimage — points in the mesh that map to the curve.

use crate::data::{CellArray, Points, PolyData};

/// A point in the 2D range space (f_value, g_value).
#[derive(Debug, Clone, Copy)]
pub struct RangePoint {
    pub f: f64,
    pub g: f64,
}

impl RangePoint {
    pub fn new(f: f64, g: f64) -> Self {
        Self { f, g }
    }
}

/// Extract a fiber surface from a PolyData with bivariate scalar fields.
///
/// # Arguments
/// * `input` - PolyData with triangle cells
/// * `f_name` - name of the first scalar array
/// * `g_name` - name of the second scalar array
/// * `fiber_curve` - sequence of (f, g) points defining the fiber curve in range space
///
/// For each segment of the fiber curve, finds triangles where both scalars
/// bracket the segment values and interpolates to find fiber points.
///
/// Returns a PolyData with line cells tracing the fiber surface.
pub fn fiber_surface(
    input: &PolyData,
    f_name: &str,
    g_name: &str,
    fiber_curve: &[RangePoint],
) -> PolyData {
    let n = input.points.len();
    if n == 0 || fiber_curve.len() < 2 {
        return PolyData::new();
    }

    // Read scalar values
    let f_arr = match input.point_data().get_array(f_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let g_arr = match input.point_data().get_array(g_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let mut f_vals = vec![0.0f64; n];
    let mut g_vals = vec![0.0f64; n];
    for i in 0..n {
        let mut buf = [0.0f64];
        f_arr.tuple_as_f64(i, &mut buf);
        f_vals[i] = buf[0];
        g_arr.tuple_as_f64(i, &mut buf);
        g_vals[i] = buf[0];
    }

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    // For each fiber curve segment
    for seg_idx in 0..fiber_curve.len() - 1 {
        let rp0 = &fiber_curve[seg_idx];
        let rp1 = &fiber_curve[seg_idx + 1];

        let mut seg_points: Vec<usize> = Vec::new();

        // Check each triangle
        for cell in input.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let ids: Vec<usize> = cell.iter().map(|&x| x as usize).collect();

            // For each edge of the triangle, check if the fiber segment
            // intersects the edge's image in (f,g) space
            let num_verts = ids.len();
            for ei in 0..num_verts {
                let a = ids[ei];
                let b = ids[(ei + 1) % num_verts];

                // Edge image in range space: (f_vals[a], g_vals[a]) to (f_vals[b], g_vals[b])
                // Fiber segment: rp0 to rp1
                // Find intersection of two line segments
                if let Some(t) = line_segment_intersect(
                    f_vals[a], g_vals[a], f_vals[b], g_vals[b], rp0.f, rp0.g, rp1.f, rp1.g,
                ) {
                    // Interpolate 3D position along edge at parameter t
                    let pa = input.points.get(a);
                    let pb = input.points.get(b);
                    let pt = [
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ];
                    let idx = out_points.len();
                    out_points.push(pt);
                    seg_points.push(idx);
                }
            }
        }

        // Connect segment points as a polyline
        if seg_points.len() >= 2 {
            let cell: Vec<i64> = seg_points.iter().map(|&i| i as i64).collect();
            out_lines.push_cell(&cell);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

/// Find the intersection parameter t along segment (ax,ay)-(bx,by)
/// with segment (cx,cy)-(dx,dy). Returns Some(t) in [0,1] if they intersect.
fn line_segment_intersect(
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    cx: f64,
    cy: f64,
    dx: f64,
    dy: f64,
) -> Option<f64> {
    let denom = (bx - ax) * (dy - cy) - (by - ay) * (dx - cx);
    if denom.abs() < 1e-12 {
        return None;
    }
    let t = ((cx - ax) * (dy - cy) - (cy - ay) * (dx - cx)) / denom;
    let u = ((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) / denom;
    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some(t)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn test_fiber_surface_basic() {
        // Create a simple triangle with bivariate scalars
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // f: 0, 1, 0; g: 0, 0, 1
        let f = DataArray::from_vec("f", vec![0.0f64, 1.0, 0.0], 1);
        let g = DataArray::from_vec("g", vec![0.0f64, 0.0, 1.0], 1);
        pd.point_data_mut().add_array(f.into());
        pd.point_data_mut().add_array(g.into());

        // Fiber curve crossing through range space
        let curve = vec![RangePoint::new(0.5, -0.5), RangePoint::new(0.5, 1.5)];

        let result = fiber_surface(&pd, "f", "g", &curve);
        // Should find some intersection points
        assert!(result.points.len() > 0, "Should find fiber points");
    }
}
