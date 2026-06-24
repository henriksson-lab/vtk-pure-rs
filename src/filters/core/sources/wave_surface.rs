//! Parametric wave surface geometry sources.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate a sinusoidal wave surface in the XY plane.
pub fn sine_wave_surface(
    x_extent: f64,
    y_extent: f64,
    amplitude: f64,
    wavelength: f64,
    resolution: usize,
) -> PolyData {
    let n = resolution.max(4);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut height_data = Vec::new();

    for j in 0..=n {
        for i in 0..=n {
            let x = x_extent * i as f64 / n as f64 - x_extent / 2.0;
            let y = y_extent * j as f64 / n as f64 - y_extent / 2.0;
            let z = amplitude
                * (2.0 * std::f64::consts::PI * x / wavelength).sin()
                * (2.0 * std::f64::consts::PI * y / wavelength).cos();
            points.push([x, y, z]);
            height_data.push(z);
        }
    }

    let row = n + 1;
    for j in 0..n {
        for i in 0..n {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Height",
            height_data,
            1,
        )));
    mesh
}

/// Generate a ripple pattern (concentric circular waves).
pub fn ripple_surface(radius: f64, amplitude: f64, wavelength: f64, resolution: usize) -> PolyData {
    let n = resolution.max(4);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for j in 0..=n {
        for i in 0..=n {
            let x = 2.0 * radius * i as f64 / n as f64 - radius;
            let y = 2.0 * radius * j as f64 / n as f64 - radius;
            let r = (x * x + y * y).sqrt();
            let z = amplitude
                * (2.0 * std::f64::consts::PI * r / wavelength).sin()
                * (-r / radius).exp().max(0.01);
            points.push([x, y, z]);
        }
    }

    let row = n + 1;
    for j in 0..n {
        for i in 0..n {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_wave() {
        let w = sine_wave_surface(4.0, 4.0, 0.5, 2.0, 16);
        assert_eq!(w.points.len(), 17 * 17);
        assert!(w.point_data().get_array("Height").is_some());
    }

    #[test]
    fn ripple() {
        let r = ripple_surface(3.0, 0.5, 1.0, 20);
        assert!(r.points.len() > 100);
        assert!(r.polys.num_cells() > 100);
    }
}
