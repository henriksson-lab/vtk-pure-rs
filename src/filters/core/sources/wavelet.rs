use crate::data::{AnyDataArray, DataArray, ImageData};

/// Parameters for generating a wavelet scalar field on ImageData.
pub struct WaveletParams {
    /// Dimensions of the output grid. Default: [20, 20, 20]
    pub dimensions: [usize; 3],
    /// Center of the wavelet. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Maximum value. Default: 10.0
    pub maximum: f64,
    /// Spatial extent (half-width). Default: 10.0
    pub extent: f64,
    /// Frequency factor. Default: 60.0
    pub x_freq: f64,
    pub y_freq: f64,
    pub z_freq: f64,
}

impl Default for WaveletParams {
    fn default() -> Self {
        Self {
            dimensions: [20, 20, 20],
            center: [0.0, 0.0, 0.0],
            maximum: 10.0,
            extent: 10.0,
            x_freq: 60.0,
            y_freq: 30.0,
            z_freq: 40.0,
        }
    }
}

/// Generate a wavelet scalar field on ImageData.
///
/// Creates a smooth scalar field with multiple frequency components,
/// useful for testing isosurface extraction and volume visualization.
/// The field is: `max * exp(-r²/extent²) * cos(xf*x) * cos(yf*y) * cos(zf*z)`
pub fn wavelet(params: &WaveletParams) -> ImageData {
    let nx = params.dimensions[0].max(2);
    let ny = params.dimensions[1].max(2);
    let nz = params.dimensions[2].max(2);

    let sp = 2.0 * params.extent / (nx as f64 - 1.0).max(1.0);
    let origin = [
        params.center[0] - params.extent,
        params.center[1] - params.extent,
        params.center[2] - params.extent,
    ];

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    img.set_origin(origin);
    img.set_spacing([sp, sp, sp]);

    let n = nx * ny * nz;
    let mut values = Vec::with_capacity(n);
    let inv_ext2 = 1.0 / (params.extent * params.extent);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let x = origin[0] + i as f64 * sp - params.center[0];
                let y = origin[1] + j as f64 * sp - params.center[1];
                let z = origin[2] + k as f64 * sp - params.center[2];
                let r2 = x * x + y * y + z * z;
                let v = params.maximum
                    * (-r2 * inv_ext2).exp()
                    * (params.x_freq * x * 0.01).cos()
                    * (params.y_freq * y * 0.01).cos()
                    * (params.z_freq * z * 0.01).cos();
                values.push(v);
            }
        }
    }

    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RTData", values, 1)));
    img.point_data_mut().set_active_scalars("RTData");
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_wavelet() {
        let img = wavelet(&WaveletParams::default());
        assert_eq!(img.dimensions(), [20, 20, 20]);
        assert!(img.point_data().get_array("RTData").is_some());
    }

    #[test]
    fn center_has_max() {
        let params = WaveletParams {
            dimensions: [11, 11, 11],
            x_freq: 0.0,
            y_freq: 0.0,
            z_freq: 0.0,
            ..Default::default()
        };
        let img = wavelet(&params);
        let arr = img.point_data().get_array("RTData").unwrap();
        let mut buf = [0.0f64];
        // Center voxel (5,5,5) = index 665
        let center = 5 * 11 * 11 + 5 * 11 + 5;
        arr.tuple_as_f64(center, &mut buf);
        assert!((buf[0] - 10.0).abs() < 0.5);
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
