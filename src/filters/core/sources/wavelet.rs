use crate::data::{AnyDataArray, DataArray, ImageData};

/// Parameters for generating a `vtkRTAnalyticSource`-style scalar field on ImageData.
pub struct WaveletParams {
    /// Whole extent of the output image. Default: [-10, 10, -10, 10, -10, 10]
    pub whole_extent: [i64; 6],
    /// Center of the wavelet. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Maximum value. Default: 255.0
    pub maximum: f64,
    /// Standard deviation of the Gaussian. Default: 0.5
    pub standard_deviation: f64,
    /// Natural frequency in x/y/z. Defaults: 60, 30, 40
    pub x_freq: f64,
    pub y_freq: f64,
    pub z_freq: f64,
    /// Magnitude in x/y/z. Defaults: 10, 18, 5
    pub x_mag: f64,
    pub y_mag: f64,
    pub z_mag: f64,
    /// Subsample rate. Default: 1
    pub subsample_rate: i64,
}

impl Default for WaveletParams {
    fn default() -> Self {
        Self {
            whole_extent: [-10, 10, -10, 10, -10, 10],
            center: [0.0, 0.0, 0.0],
            maximum: 255.0,
            standard_deviation: 0.5,
            x_freq: 60.0,
            y_freq: 30.0,
            z_freq: 40.0,
            x_mag: 10.0,
            y_mag: 18.0,
            z_mag: 5.0,
            subsample_rate: 1,
        }
    }
}

/// Generate a wavelet scalar field on ImageData.
///
/// Creates the same analytic field as VTK's `vtkRTAnalyticSource`:
/// `Maximum*Gaussian + XMag*sin(XFreq*x) + YMag*sin(YFreq*y) + ZMag*cos(ZFreq*z)`.
pub fn wavelet(params: &WaveletParams) -> ImageData {
    let subsample_rate = params.subsample_rate;
    let whole_extent = params.whole_extent;
    let mut img = ImageData::new();

    if subsample_rate <= 0
        || whole_extent[0] > whole_extent[1]
        || whole_extent[2] > whole_extent[3]
        || whole_extent[4] > whole_extent[5]
    {
        return img;
    }

    for axis in 0..3 {
        debug_assert!(whole_extent[2 * axis] <= whole_extent[2 * axis + 1]);
    }

    let output_extent = [
        whole_extent[0] / subsample_rate,
        whole_extent[1] / subsample_rate,
        whole_extent[2] / subsample_rate,
        whole_extent[3] / subsample_rate,
        whole_extent[4] / subsample_rate,
        whole_extent[5] / subsample_rate,
    ];
    let nx = (output_extent[1] - output_extent[0] + 1).max(0) as usize;
    let ny = (output_extent[3] - output_extent[2] + 1).max(0) as usize;
    let nz = (output_extent[5] - output_extent[4] + 1).max(0) as usize;

    img.set_extent(output_extent);
    img.set_origin([0.0, 0.0, 0.0]);
    img.set_spacing([
        subsample_rate as f64,
        subsample_rate as f64,
        subsample_rate as f64,
    ]);

    let n = nx * ny * nz;
    let mut values = Vec::with_capacity(n);
    let temp2 = 1.0 / (2.0 * params.standard_deviation * params.standard_deviation);
    let xscale = if whole_extent[1] > whole_extent[0] {
        1.0 / (whole_extent[1] - whole_extent[0]) as f64
    } else {
        1.0
    };
    let yscale = if whole_extent[3] > whole_extent[2] {
        1.0 / (whole_extent[3] - whole_extent[2]) as f64
    } else {
        1.0
    };
    let zscale = if whole_extent[5] > whole_extent[4] {
        1.0 / (whole_extent[5] - whole_extent[4]) as f64
    } else {
        1.0
    };

    for k in 0..nz {
        let idx_z = k as i64 * subsample_rate;
        let z = (params.center[2] - (idx_z + whole_extent[4]) as f64) * zscale;
        let z_contrib = z * z;
        let z_factor = params.z_mag * (params.z_freq * z).cos();
        for j in 0..ny {
            let idx_y = j as i64 * subsample_rate;
            let y = (params.center[1] - (idx_y + whole_extent[2]) as f64) * yscale;
            let y_contrib = y * y;
            let y_factor = params.y_mag * (params.y_freq * y).sin();
            for i in 0..nx {
                let idx_x = i as i64 * subsample_rate;
                let x = (params.center[0] - (idx_x + whole_extent[0]) as f64) * xscale;
                let sum = z_contrib + y_contrib + x * x;
                let x_factor = params.x_mag * (params.x_freq * x).sin();
                values.push(
                    (params.maximum * (-sum * temp2).exp() + x_factor + y_factor + z_factor) as f32,
                );
            }
        }
    }

    img.point_data_mut()
        .add_array(AnyDataArray::F32(DataArray::from_vec("RTData", values, 1)));
    img.point_data_mut().set_active_scalars("RTData");
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_wavelet() {
        let img = wavelet(&WaveletParams::default());
        assert_eq!(img.dimensions(), [21, 21, 21]);
        assert_eq!(img.extent(), [-10, 10, -10, 10, -10, 10]);
        assert!(img.point_data().get_array("RTData").is_some());
    }

    #[test]
    fn center_has_max() {
        let params = WaveletParams {
            whole_extent: [-5, 5, -5, 5, -5, 5],
            x_freq: 0.0,
            y_freq: 0.0,
            z_freq: 0.0,
            x_mag: 0.0,
            y_mag: 0.0,
            z_mag: 0.0,
            ..Default::default()
        };
        let img = wavelet(&params);
        let arr = img.point_data().get_array("RTData").unwrap();
        let mut buf = [0.0f64];
        // Center voxel (5,5,5) = index 665
        let center = 5 * 11 * 11 + 5 * 11 + 5;
        arr.tuple_as_f64(center, &mut buf);
        assert!((buf[0] - 255.0).abs() < 0.5);
    }

    #[test]
    fn has_variation() {
        let img = wavelet(&WaveletParams::default());
        let arr = img.point_data().get_array("RTData").unwrap();
        let mut buf = [0.0f64];
        let mut min_v = f64::MAX;
        let mut max_v = f64::MIN;
        for i in 0..20 * 20 * 20 {
            arr.tuple_as_f64(i, &mut buf);
            min_v = min_v.min(buf[0]);
            max_v = max_v.max(buf[0]);
        }
        assert!(max_v > min_v);
    }
}
