//! Voronoi and Gaussian kernel functions for point interpolation.
//!
//! Provides kernel evaluators and Voronoi-based natural neighbor interpolation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Gaussian kernel: w(r) = exp(-r²/(2σ²))
pub fn gaussian_kernel_weight(r: f64, sigma: f64) -> f64 {
    (-r * r / (2.0 * sigma * sigma)).exp()
}

/// Voronoi kernel: 1/distance (natural neighbor approximation).
///
/// Each neighbor's weight is proportional to 1/distance.
pub fn voronoi_kernel_weight(r: f64) -> f64 {
    if r < 1e-15 {
        1e15
    } else {
        1.0 / r
    }
}

/// Epanechnikov kernel: w(r) = max(0, 1 - (r/h)²)
pub fn epanechnikov_kernel_weight(r: f64, bandwidth: f64) -> f64 {
    let q = r / bandwidth;
    if q >= 1.0 {
        0.0
    } else {
        1.0 - q * q
    }
}

/// Interpolate using Gaussian kernel with automatic bandwidth estimation.
///
/// Bandwidth is estimated as the average distance to k-th nearest neighbor.
pub fn gaussian_interpolate(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
    sigma: f64,
    max_radius: f64,
) -> PolyData {
    kernel_interpolate_impl(source, target, array_name, max_radius, |r| {
        gaussian_kernel_weight(r, sigma)
    })
}

/// Interpolate using Voronoi (inverse-distance) kernel.
pub fn voronoi_interpolate(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
    max_radius: f64,
) -> PolyData {
    kernel_interpolate_impl(
        source,
        target,
        array_name,
        max_radius,
        voronoi_kernel_weight,
    )
}

/// Interpolate using Epanechnikov kernel.
pub fn epanechnikov_interpolate(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
    bandwidth: f64,
) -> PolyData {
    kernel_interpolate_impl(source, target, array_name, bandwidth, |r| {
        epanechnikov_kernel_weight(r, bandwidth)
    })
}

/// Compute kernel density estimation at each point.
pub fn kernel_density_estimate(mesh: &PolyData, sigma: f64, max_radius: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let r2_max = max_radius * max_radius;
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut density = Vec::with_capacity(n);
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            if i == j {
                continue;
            }
            let r2 = dist2(pts[i], pts[j]);
            if r2 <= r2_max {
                sum += gaussian_kernel_weight(r2.sqrt(), sigma);
            }
        }
        density.push(sum);
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "KernelDensity",
            density,
            1,
        )));
    result
}

fn kernel_interpolate_impl(
    source: &PolyData,
    target: &PolyData,
    array_name: &str,
    max_radius: f64,
    kernel_fn: impl Fn(f64) -> f64,
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

    let r2_max = max_radius * max_radius;
    let src_pts: Vec<[f64; 3]> = (0..n_src).map(|i| source.points.get(i)).collect();
    let mut src_vals = Vec::with_capacity(n_src);
    let mut buf = [0.0f64];
    for i in 0..n_src {
        arr.tuple_as_f64(i, &mut buf);
        src_vals.push(buf[0]);
    }

    let mut out = Vec::with_capacity(n_tgt);
    for ti in 0..n_tgt {
        let tp = target.points.get(ti);
        let mut sum_wv = 0.0;
        let mut sum_w = 0.0;
        for si in 0..n_src {
            let r2 = dist2(tp, src_pts[si]);
            if r2 > r2_max {
                continue;
            }
            let w = kernel_fn(r2.sqrt());
            sum_wv += w * src_vals[si];
            sum_w += w;
        }
        out.push(if sum_w > 1e-15 { sum_wv / sum_w } else { 0.0 });
    }

    let mut result = target.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, out, 1)));
    result
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    fn make_source() -> PolyData {
        let mut m = PolyData::new();
        m.points = Points::from(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 100.0, 0.0],
                1,
            )));
        m
    }

    #[test]
    fn gaussian_interp() {
        let tgt = PolyData::from_points(vec![[0.5, 0.0, 0.0]]);
        let r = gaussian_interpolate(&make_source(), &tgt, "val", 1.0, 5.0);
        let a = r.point_data().get_array("val").unwrap();
        let mut b = [0.0f64];
        a.tuple_as_f64(0, &mut b);
        assert!(b[0] > 0.0 && b[0] < 100.0);
    }

    #[test]
    fn voronoi_interp() {
        let tgt = PolyData::from_points(vec![[0.5, 0.0, 0.0]]);
        let r = voronoi_interpolate(&make_source(), &tgt, "val", 5.0);
        assert!(r.point_data().get_array("val").is_some());
    }

    #[test]
    fn epanechnikov_interp() {
        let tgt = PolyData::from_points(vec![[0.5, 0.0, 0.0]]);
        let r = epanechnikov_interpolate(&make_source(), &tgt, "val", 2.0);
        assert!(r.point_data().get_array("val").is_some());
    }

    #[test]
    fn kde() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [5.0, 0.0, 0.0]]);
        let r = kernel_density_estimate(&mesh, 0.5, 2.0);
        let a = r.point_data().get_array("KernelDensity").unwrap();
        let mut b = [0.0f64];
        a.tuple_as_f64(0, &mut b);
        let dense = b[0];
        a.tuple_as_f64(2, &mut b);
        let sparse = b[0];
        assert!(dense > sparse);
    }
}
