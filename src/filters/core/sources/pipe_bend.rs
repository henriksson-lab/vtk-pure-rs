//! Pipe bend (elbow) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a pipe bend (elbow) with given bend angle and radii.
pub fn pipe_bend(
    bend_radius: f64,
    pipe_radius: f64,
    angle_degrees: f64,
    bend_res: usize,
    tube_res: usize,
) -> PolyData {
    let bres = bend_res.max(2);
    let tres = tube_res.max(3);
    let angle = angle_degrees.to_radians();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for ib in 0..=bres {
        let t = ib as f64 / bres as f64;
        let a = angle * t;
        // Center of pipe cross-section follows arc in XZ plane
        let cx = bend_radius * a.sin();
        let cy = 0.0;
        let cz = bend_radius * (1.0 - a.cos());

        // Tangent direction along the bend
        let tx = a.cos();
        let tz = a.sin();
        // Radial direction (pointing away from bend center)
        let _rx = a.sin();
        let rz = -(1.0 - a.cos()).signum() * a.cos();
        let _ = rz; // We use a simpler frame

        // Normal to tangent in XZ plane
        let nx = -tz;
        let nz = tx;

        for it in 0..tres {
            let phi = 2.0 * std::f64::consts::PI * it as f64 / tres as f64;
            let c = phi.cos();
            let s = phi.sin();
            pts.push([
                cx + pipe_radius * (c * nx),
                cy + pipe_radius * s,
                cz + pipe_radius * (c * nz),
            ]);
        }
    }

    for ib in 0..bres {
        let r0 = ib * tres;
        let r1 = (ib + 1) * tres;
        for it in 0..tres {
            let it1 = (it + 1) % tres;
            polys.push_cell(&[
                (r0 + it) as i64,
                (r0 + it1) as i64,
                (r1 + it1) as i64,
                (r1 + it) as i64,
            ]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_90_degree() {
        let p = pipe_bend(2.0, 0.5, 90.0, 16, 12);
        assert_eq!(p.points.len(), 17 * 12);
        assert_eq!(p.polys.num_cells(), 16 * 12);
    }
    #[test]
    fn test_180_degree() {
        let p = pipe_bend(3.0, 0.3, 180.0, 24, 8);
        assert_eq!(p.points.len(), 25 * 8);
    }
}
