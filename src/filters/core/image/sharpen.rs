use crate::data::{AnyDataArray, DataArray, ImageData};

/// Unsharp mask sharpening on ImageData.
///
/// Computes `sharpened = original + alpha * (original - blurred)` where
/// `blurred` is a box blur with the given `radius` (half-size in voxels).
/// Adds a "Sharpened" array to the output point data.
pub fn unsharp_mask(input: &ImageData, scalars: &str, radius: usize, alpha: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n: usize = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    // Box blur (separable, 3 passes)
    let r = radius.max(1);

    // X pass
    let mut tmp = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut acc: f64 = 0.0;
                let mut cnt: f64 = 0.0;
                let lo = if i >= r { i - r } else { 0 };
                let hi = (i + r).min(nx - 1);
                for ii in lo..=hi {
                    acc += values[k * ny * nx + j * nx + ii];
                    cnt += 1.0;
                }
                tmp[k * ny * nx + j * nx + i] = acc / cnt;
            }
        }
    }

    // Y pass
    let mut tmp2 = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut acc: f64 = 0.0;
                let mut cnt: f64 = 0.0;
                let lo = if j >= r { j - r } else { 0 };
                let hi = (j + r).min(ny - 1);
                for jj in lo..=hi {
                    acc += tmp[k * ny * nx + jj * nx + i];
                    cnt += 1.0;
                }
                tmp2[k * ny * nx + j * nx + i] = acc / cnt;
            }
        }
    }

    // Z pass
    let mut blurred = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut acc: f64 = 0.0;
                let mut cnt: f64 = 0.0;
                let lo = if k >= r { k - r } else { 0 };
                let hi = (k + r).min(nz - 1);
                for kk in lo..=hi {
                    acc += tmp2[kk * ny * nx + j * nx + i];
                    cnt += 1.0;
                }
                blurred[k * ny * nx + j * nx + i] = acc / cnt;
            }
        }
    }

    // Unsharp mask: original + alpha * (original - blurred)
    let mut sharpened = vec![0.0f64; n];
    for i in 0..n {
        sharpened[i] = values[i] + alpha * (values[i] - blurred[i]);
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Sharpened",
            sharpened,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spike_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n: usize = 125;
        let mut values = vec![0.0f64; n];
        values[62] = 100.0; // spike at center (2,2,2)
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn sharpening_increases_spike() {
        let img = make_spike_image();
        let result = unsharp_mask(&img, "val", 1, 1.0);
        let arr = result.point_data().get_array("Sharpened").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(62, &mut buf);
        // Sharpening should increase the spike above the original 100
        assert!(buf[0] > 100.0, "center sharpened={}", buf[0]);
    }

    #[test]
    fn uniform_image_unchanged() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![5.0f64; 27],
                1,
            )));
        let result = unsharp_mask(&img, "val", 1, 2.0);
        let arr = result.point_data().get_array("Sharpened").unwrap();
        let mut buf = [0.0f64];
        for i in 0..27 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 5.0).abs() < 1e-10, "voxel {} = {}", i, buf[0]);
        }
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let result = unsharp_mask(&img, "nonexistent", 1, 1.0);
        assert!(result.point_data().get_array("Sharpened").is_none());
    }
}
