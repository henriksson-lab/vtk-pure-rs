//! SPH (Smoothed Particle Hydrodynamics) kernel interpolation.
//!
//! Interpolates data from particles using SPH kernels (cubic spline,
//! Wendland, etc.) with density-weighted averaging.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// SPH kernel type.
#[derive(Debug, Clone, Copy)]
pub enum SphKernel {
    /// Cubic spline (M4) kernel.
    CubicSpline,
    /// Wendland C2 kernel.
    WendlandC2,
    /// Quintic spline kernel.
    QuinticSpline,
}

/// Interpolate scalar data from source particles to target points using SPH.
///
/// `smoothing_length` controls the kernel radius (support radius = 2h for cubic spline).
pub fn sph_interpolate(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
    smoothing_length: f64,
    kernel: SphKernel,
) -> PolyData {
    let n_src = source.points.len();
    let n_tgt = target.points.len();
    if n_src == 0 || n_tgt == 0 {
        return target.clone();
    }

    let arr = match source.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return target.clone(),
    };

    let h = smoothing_length;
    let support = 2.0 * h;
    let support2 = support * support;

    let src_pts: Vec<[f64; 3]> = (0..n_src).map(|i| source.points.get(i)).collect();
    let mut src_vals = Vec::with_capacity(n_src);
    let mut buf = [0.0f64];
    for i in 0..n_src {
        arr.tuple_as_f64(i, &mut buf);
        src_vals.push(buf[0]);
    }

    let mut result_vals = Vec::with_capacity(n_tgt);
    for ti in 0..n_tgt {
        let tp = target.points.get(ti);
        let mut sum_wv = 0.0;
        let mut sum_w = 0.0;

        for si in 0..n_src {
            let sp = &src_pts[si];
            let dx = tp[0] - sp[0];
            let dy = tp[1] - sp[1];
            let dz = tp[2] - sp[2];
            let r2 = dx * dx + dy * dy + dz * dz;
            if r2 > support2 {
                continue;
            }
            let r = r2.sqrt();
            let w = eval_kernel(kernel, r, h);
            sum_wv += w * src_vals[si];
            sum_w += w;
        }

        result_vals.push(if sum_w > 1e-15 { sum_wv / sum_w } else { 0.0 });
    }

    let mut result = target.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            result_vals,
            1,
        )));
    result
}

/// Compute SPH density estimate at each particle.
pub fn sph_density(mesh: &PolyData, smoothing_length: f64, kernel: SphKernel) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let h = smoothing_length;
    let support2 = (2.0 * h) * (2.0 * h);
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut density = Vec::with_capacity(n);
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            let dx = pts[i][0] - pts[j][0];
            let dy = pts[i][1] - pts[j][1];
            let dz = pts[i][2] - pts[j][2];
            let r2 = dx * dx + dy * dy + dz * dz;
            if r2 <= support2 {
                sum += eval_kernel(kernel, r2.sqrt(), h);
            }
        }
        density.push(sum);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SPHDensity",
            density,
            1,
        )));
    result
}

fn eval_kernel(kernel: SphKernel, r: f64, h: f64) -> f64 {
    let q = r / h;
    match kernel {
        SphKernel::CubicSpline => {
            let norm = 1.0 / (std::f64::consts::PI * h * h * h);
            if q < 1.0 {
                norm * (1.0 - 1.5 * q * q + 0.75 * q * q * q)
            } else if q < 2.0 {
                norm * 0.25 * (2.0 - q).powi(3)
            } else {
                0.0
            }
        }
        SphKernel::WendlandC2 => {
            let norm = 21.0 / (16.0 * std::f64::consts::PI * h * h * h);
            if q < 2.0 {
                let t = 1.0 - q * 0.5;
                norm * t.powi(4) * (1.0 + 2.0 * q)
            } else {
                0.0
            }
        }
        SphKernel::QuinticSpline => {
            let norm = 1.0 / (120.0 * std::f64::consts::PI * h * h * h);
            if q < 1.0 {
                norm * ((3.0 - q).powi(5) - 6.0 * (2.0 - q).powi(5) + 15.0 * (1.0 - q).powi(5))
            } else if q < 2.0 {
                norm * ((3.0 - q).powi(5) - 6.0 * (2.0 - q).powi(5))
            } else if q < 3.0 {
                norm * (3.0 - q).powi(5)
            } else {
                0.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    #[test]
    fn cubic_spline_interpolation() {
        let mut source = PolyData::new();
        source.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        source
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![0.0, 100.0, 0.0],
                1,
            )));

        let mut target = PolyData::new();
        target.points = Points::from(vec![[0.5, 0.0, 0.0]]);

        let result = sph_interpolate(&source, &target, "temp", 1.0, SphKernel::CubicSpline);
        let arr = result.point_data().get_array("temp").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0 && buf[0] < 100.0);
    }

    #[test]
    fn wendland_density() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from(vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.2, 0.0, 0.0],
            [5.0, 0.0, 0.0],
        ]);
        let result = sph_density(&mesh, 0.5, SphKernel::WendlandC2);
        let arr = result.point_data().get_array("SPHDensity").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let dense = buf[0];
        arr.tuple_as_f64(3, &mut buf);
        let sparse = buf[0];
        assert!(dense > sparse, "dense={dense}, sparse={sparse}");
    }

    #[test]
    fn empty() {
        let result = sph_interpolate(
            &PolyData::new(),
            &PolyData::new(),
            "x",
            1.0,
            SphKernel::CubicSpline,
        );
        assert_eq!(result.points.len(), 0);
    }
}
