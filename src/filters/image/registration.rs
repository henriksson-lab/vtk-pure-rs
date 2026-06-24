//! Image registration: align two ImageData volumes by translation.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the optimal translation to align `moving` to `fixed` using
/// normalized cross-correlation (NCC) on scalar arrays.
///
/// Returns the translation [dx, dy, dz] in voxel units.
pub fn register_translation_3d(
    fixed: &ImageData,
    moving: &ImageData,
    array_name: &str,
    search_radius: usize,
) -> [i64; 3] {
    let f_arr = match fixed.point_data().get_array(array_name) {
        Some(a) => a,
        None => return [0; 3],
    };
    let m_arr = match moving.point_data().get_array(array_name) {
        Some(a) => a,
        None => return [0; 3],
    };

    let f_dims = fixed.dimensions();
    let m_dims = moving.dimensions();
    let r = search_radius as i64;

    let mut best_corr = f64::MIN;
    let mut best_shift = [0i64; 3];
    let mut f_buf = [0.0f64];
    let mut m_buf = [0.0f64];

    for dz in -r..=r {
        for dy in -r..=r {
            for dx in -r..=r {
                let mut sum_fm = 0.0;
                let mut sum_ff = 0.0;
                let mut sum_mm = 0.0;
                let mut count = 0;

                let zr = 0..f_dims[2].min(m_dims[2]);
                let yr = 0..f_dims[1].min(m_dims[1]);
                let xr = 0..f_dims[0].min(m_dims[0]);

                for iz in zr {
                    for iy in yr.clone() {
                        for ix in xr.clone() {
                            let mx = ix as i64 + dx;
                            let my = iy as i64 + dy;
                            let mz = iz as i64 + dz;
                            if mx < 0 || my < 0 || mz < 0 {
                                continue;
                            }
                            let mx = mx as usize;
                            let my = my as usize;
                            let mz = mz as usize;
                            if mx >= m_dims[0] || my >= m_dims[1] || mz >= m_dims[2] {
                                continue;
                            }

                            let fi = ix + iy * f_dims[0] + iz * f_dims[0] * f_dims[1];
                            let mi = mx + my * m_dims[0] + mz * m_dims[0] * m_dims[1];

                            if fi >= f_arr.num_tuples() || mi >= m_arr.num_tuples() {
                                continue;
                            }

                            f_arr.tuple_as_f64(fi, &mut f_buf);
                            m_arr.tuple_as_f64(mi, &mut m_buf);

                            sum_fm += f_buf[0] * m_buf[0];
                            sum_ff += f_buf[0] * f_buf[0];
                            sum_mm += m_buf[0] * m_buf[0];
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let denom = (sum_ff * sum_mm).sqrt();
                    let ncc = if denom > 1e-15 { sum_fm / denom } else { 0.0 };
                    if ncc > best_corr {
                        best_corr = ncc;
                        best_shift = [dx, dy, dz];
                    }
                }
            }
        }
    }

    best_shift
}

/// Apply a voxel-unit translation to an ImageData by shifting the origin.
pub fn apply_translation(image: &ImageData, shift: [i64; 3]) -> ImageData {
    let origin = image.origin();
    let spacing = image.spacing();
    let mut result = image.clone();
    result.set_origin([
        origin[0] + shift[0] as f64 * spacing[0],
        origin[1] + shift[1] as f64 * spacing[1],
        origin[2] + shift[2] as f64 * spacing[2],
    ]);
    result
}

/// Compute the NCC similarity between two aligned images.
pub fn ncc_similarity(a: &ImageData, b: &ImageData, array_name: &str) -> f64 {
    let a_arr = match a.point_data().get_array(array_name) {
        Some(x) => x,
        None => return 0.0,
    };
    let b_arr = match b.point_data().get_array(array_name) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = a_arr.num_tuples().min(b_arr.num_tuples());
    if n == 0 {
        return 0.0;
    }

    let mut sum_ab = 0.0;
    let mut sum_aa = 0.0;
    let mut sum_bb = 0.0;
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    for i in 0..n {
        a_arr.tuple_as_f64(i, &mut ab);
        b_arr.tuple_as_f64(i, &mut bb);
        sum_ab += ab[0] * bb[0];
        sum_aa += ab[0] * ab[0];
        sum_bb += bb[0] * bb[0];
    }
    let denom = (sum_aa * sum_bb).sqrt();
    if denom > 1e-15 {
        sum_ab / denom
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images() {
        let img = ImageData::from_function(
            [10, 10, 10],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, z| (x * 1.3).sin() * (y * 0.7).cos() * (z * 1.1 + 0.5).sin(),
        );
        let shift = register_translation_3d(&img, &img, "val", 2);
        assert_eq!(shift, [0, 0, 0]);
    }

    #[test]
    fn ncc_identical() {
        let img = ImageData::from_function(
            [5, 5, 5],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _, _| x,
        );
        let ncc = ncc_similarity(&img, &img, "val");
        assert!((ncc - 1.0).abs() < 0.01);
    }

    #[test]
    fn apply_shift() {
        let img = ImageData::with_dimensions(5, 5, 5).with_spacing([1.0, 1.0, 1.0]);
        let shifted = apply_translation(&img, [2, 3, 1]);
        let o = shifted.origin();
        assert!((o[0] - 2.0).abs() < 1e-10);
        assert!((o[1] - 3.0).abs() < 1e-10);
    }
}
