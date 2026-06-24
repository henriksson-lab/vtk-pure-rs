//! Stencil-based region processing on ImageData.
//!
//! A stencil defines a binary mask that selects which voxels to process.
//! Operations: apply stencil, create stencil from threshold, invert stencil.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a binary stencil to ImageData. Voxels outside the stencil are set to `fill_value`.
///
/// `stencil` is a 1-component array where nonzero = inside.
pub fn apply_stencil(
    input: &ImageData,
    array_name: &str,
    stencil_name: &str,
    fill_value: f64,
) -> ImageData {
    let dims = input.dimensions();
    let n = dims[0] * dims[1] * dims[2];

    let scalars = match input.point_data().get_array(array_name) {
        Some(s) => s,
        None => return input.clone(),
    };
    let stencil = match input.point_data().get_array(stencil_name) {
        Some(s) => s,
        None => return input.clone(),
    };

    let mut result_data = Vec::with_capacity(n);
    for i in 0..n {
        let mut sv = [0.0f64];
        stencil.tuple_as_f64(i, &mut sv);
        if sv[0].abs() > 1e-30 {
            let mut v = [0.0f64];
            scalars.tuple_as_f64(i, &mut v);
            result_data.push(v[0]);
        } else {
            result_data.push(fill_value);
        }
    }

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            result_data,
            1,
        )));
    output
}

/// Create a binary stencil from a scalar threshold.
/// Voxels where scalar >= threshold are set to 1.0, others to 0.0.
pub fn create_stencil_from_threshold(
    input: &ImageData,
    array_name: &str,
    threshold: f64,
) -> ImageData {
    let dims = input.dimensions();
    let n = dims[0] * dims[1] * dims[2];

    let scalars = match input.point_data().get_array(array_name) {
        Some(s) => s,
        None => return input.clone(),
    };

    let mut stencil = Vec::with_capacity(n);
    for i in 0..n {
        let mut v = [0.0f64];
        scalars.tuple_as_f64(i, &mut v);
        stencil.push(if v[0] >= threshold { 1.0 } else { 0.0 });
    }

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Stencil", stencil, 1,
        )));
    output
}

/// Window/level → RGBA mapping for medical imaging.
///
/// Maps scalar values through a window/level transfer function:
/// - Values below (level - window/2) → 0.0
/// - Values above (level + window/2) → 1.0
/// - Values in between → linear interpolation
/// Output: 4-component RGBA array.
pub fn window_level_to_rgba(
    input: &ImageData,
    array_name: &str,
    window: f64,
    level: f64,
) -> ImageData {
    let dims = input.dimensions();
    let n = dims[0] * dims[1] * dims[2];

    let scalars = match input.point_data().get_array(array_name) {
        Some(s) => s,
        None => return input.clone(),
    };

    let low = level - window / 2.0;
    let high = level + window / 2.0;
    let range = high - low;

    let mut rgba = Vec::with_capacity(n * 4);
    for i in 0..n {
        let mut v = [0.0f64];
        scalars.tuple_as_f64(i, &mut v);
        let t = if range.abs() < 1e-30 {
            0.5
        } else {
            ((v[0] - low) / range).clamp(0.0, 1.0)
        };
        rgba.push(t);
        rgba.push(t);
        rgba.push(t);
        rgba.push(1.0);
    }

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("RGBA", rgba, 4)));
    output
}

/// Frequency-domain ideal low-pass filter.
/// After FFT, zero out all frequencies above `cutoff_radius`.
pub fn ideal_lowpass(magnitude: &mut [f64], width: usize, height: usize, cutoff_radius: f64) {
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let r2 = cutoff_radius * cutoff_radius;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            if dx * dx + dy * dy > r2 {
                magnitude[y * width + x] = 0.0;
            }
        }
    }
}

/// Frequency-domain ideal high-pass filter.
pub fn ideal_highpass(magnitude: &mut [f64], width: usize, height: usize, cutoff_radius: f64) {
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let r2 = cutoff_radius * cutoff_radius;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            if dx * dx + dy * dy <= r2 {
                magnitude[y * width + x] = 0.0;
            }
        }
    }
}

/// Butterworth low-pass filter of given order.
/// H(u,v) = 1 / (1 + (D/D0)^(2n))
pub fn butterworth_lowpass(
    magnitude: &mut [f64],
    width: usize,
    height: usize,
    cutoff: f64,
    order: u32,
) {
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let n2 = 2 * order;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let h = 1.0 / (1.0 + (d / cutoff).powi(n2 as i32));
            magnitude[y * width + x] *= h;
        }
    }
}

/// Butterworth high-pass filter of given order.
pub fn butterworth_highpass(
    magnitude: &mut [f64],
    width: usize,
    height: usize,
    cutoff: f64,
    order: u32,
) {
    let cx = width as f64 / 2.0;
    let cy = height as f64 / 2.0;
    let n2 = 2 * order;
    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let h = if d < 1e-30 {
                0.0
            } else {
                1.0 / (1.0 + (cutoff / d).powi(n2 as i32))
            };
            magnitude[y * width + x] *= h;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> ImageData {
        let mut grid = ImageData::new([4, 4, 1], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let vals: Vec<f64> = (0..16).map(|i| i as f64).collect();
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalar", vals, 1)));
        grid
    }

    #[test]
    fn stencil_from_threshold() {
        let grid = make_grid();
        let result = create_stencil_from_threshold(&grid, "scalar", 8.0);
        let stencil = result.point_data().get_array("Stencil").unwrap();
        let mut v = [0.0f64];
        stencil.tuple_as_f64(0, &mut v);
        assert_eq!(v[0], 0.0); // 0.0 < 8.0
        stencil.tuple_as_f64(8, &mut v);
        assert_eq!(v[0], 1.0); // 8.0 >= 8.0
    }

    #[test]
    fn apply_stencil_test() {
        let mut grid = make_grid();
        let stencil_vals: Vec<f64> = (0..16).map(|i| if i >= 8 { 1.0 } else { 0.0 }).collect();
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "mask",
                stencil_vals,
                1,
            )));
        let result = apply_stencil(&grid, "scalar", "mask", -1.0);
        let arr = result.point_data().get_array("scalar").unwrap();
        let mut v = [0.0f64];
        arr.tuple_as_f64(0, &mut v);
        assert_eq!(v[0], -1.0); // masked
        arr.tuple_as_f64(10, &mut v);
        assert_eq!(v[0], 10.0); // kept
    }

    #[test]
    fn window_level() {
        let grid = make_grid();
        let result = window_level_to_rgba(&grid, "scalar", 10.0, 7.5);
        let rgba = result.point_data().get_array("RGBA").unwrap();
        // value 2.5 → level=7.5, window=10, low=2.5, high=12.5 → t=(2.5-2.5)/10=0
        let mut v = [0.0f64; 4];
        rgba.tuple_as_f64(0, &mut v); // scalar=0 → t=0 (clamped)
        assert_eq!(v[0], 0.0);
        assert_eq!(v[3], 1.0); // alpha always 1
    }

    #[test]
    fn butterworth_filter() {
        let mut data = vec![1.0; 16];
        butterworth_lowpass(&mut data, 4, 4, 1.0, 2);
        // Center should be ~1, corners should be attenuated
        assert!(data[4 * 2 + 2] > data[0]); // center > corner
    }
}
