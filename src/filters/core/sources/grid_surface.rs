//! Regular grid surface source (flat or from function).

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Create a flat grid surface in the XY plane.
pub fn grid_surface(nx: usize, ny: usize, dx: f64, dy: f64) -> PolyData {
    grid_surface_from_fn(nx, ny, dx, dy, |_, _| 0.0)
}

/// Create a grid surface with Z values from a function f(x, y).
pub fn grid_surface_from_fn(
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
    f: impl Fn(f64, f64) -> f64,
) -> PolyData {
    let nx = nx.max(2);
    let ny = ny.max(2);
    let mut pts = Points::<f64>::new();
    let mut elevations = Vec::with_capacity(nx * ny);

    for iy in 0..ny {
        for ix in 0..nx {
            let x = ix as f64 * dx;
            let y = iy as f64 * dy;
            let z = f(x, y);
            pts.push([x, y, z]);
            elevations.push(z);
        }
    }

    let mut polys = CellArray::new();
    for iy in 0..ny - 1 {
        for ix in 0..nx - 1 {
            let i00 = (iy * nx + ix) as i64;
            let i10 = (iy * nx + ix + 1) as i64;
            let i01 = ((iy + 1) * nx + ix) as i64;
            let i11 = ((iy + 1) * nx + ix + 1) as i64;
            polys.push_cell(&[i00, i10, i11, i01]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Elevation",
            elevations,
            1,
        )));
    result.point_data_mut().set_active_scalars("Elevation");
    result
}

/// Create a sine wave surface.
pub fn sine_surface(nx: usize, ny: usize, dx: f64, dy: f64, amplitude: f64, freq: f64) -> PolyData {
    grid_surface_from_fn(nx, ny, dx, dy, |x, y| {
        amplitude * (freq * x).sin() * (freq * y).sin()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flat() {
        let g = grid_surface(5, 5, 1.0, 1.0);
        assert_eq!(g.points.len(), 25);
        assert_eq!(g.polys.num_cells(), 16); // 4x4 quads
    }
    #[test]
    fn test_from_fn() {
        let g = grid_surface_from_fn(10, 10, 0.1, 0.1, |x, y| x * y);
        assert_eq!(g.points.len(), 100);
        assert!(g.point_data().get_array("Elevation").is_some());
    }
    #[test]
    fn test_sine() {
        let g = sine_surface(20, 20, 0.1, 0.1, 1.0, 5.0);
        assert_eq!(g.points.len(), 400);
    }
}
