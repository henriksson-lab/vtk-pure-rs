//! Bilateral filter for edge-preserving denoising.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply bilateral filter (edge-preserving smooth).
/// sigma_spatial controls spatial extent, sigma_range controls intensity sensitivity.
pub fn bilateral_filter(
    input: &ImageData,
    scalars: &str,
    sigma_spatial: f64,
    sigma_range: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let radius = (3.0 * sigma_spatial).ceil() as isize;
    let ss2 = 2.0 * sigma_spatial * sigma_spatial;
    let sr2 = 2.0 * sigma_range * sigma_range;

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            let center = vals[idx];
            let mut sum = 0.0;
            let mut wsum = 0.0;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let sx = ix as isize + dx;
                    let sy = iy as isize + dy;
                    if sx < 0 || sx >= nx as isize || sy < 0 || sy >= ny as isize {
                        continue;
                    }
                    let v = vals[sx as usize + sy as usize * nx];
                    let ds2 = (dx * dx + dy * dy) as f64;
                    let dr2 = (v - center) * (v - center);
                    let w = (-ds2 / ss2 - dr2 / sr2).exp();
                    sum += w * v;
                    wsum += w;
                }
            }
            if wsum > 1e-15 {
                sum / wsum
            } else {
                center
            }
        })
        .collect();

    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bilateral() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _y, _| {
                if x > 5.0 {
                    100.0
                } else {
                    0.0
                }
            },
        );
        let r = bilateral_filter(&img, "v", 2.0, 50.0);
        assert_eq!(r.dimensions(), [10, 10, 1]);
        // Edge should be preserved (value near boundary shouldn't be averaged to 50)
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(8 + 5 * 10, &mut buf);
        assert!(buf[0] > 80.0); // far from edge stays high
    }
}
