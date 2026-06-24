use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute morphological gradient (dilation - erosion) of ImageData.
///
/// Highlights edges in binary or grayscale images. The gradient is the
/// difference between the local max and local min in a neighborhood.
pub fn image_morphological_gradient(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let r = radius.max(1) as i64;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut result = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut lo = f64::MAX;
                let mut hi = f64::MIN;
                for dk in -r..=r {
                    for dj in -r..=r {
                        for di in -r..=r {
                            let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                            let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                            let v = values[kk * ny * nx + jj * nx + ii];
                            lo = lo.min(v);
                            hi = hi.max(v);
                        }
                    }
                }
                result[k * ny * nx + j * nx + i] = hi - lo;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorphGradient",
            result,
            1,
        )));
    img
}

/// Morphological opening: erosion followed by dilation. Removes small bright features.
pub fn image_morphological_opening(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let eroded = crate::filters::image::dilate_erode::image_erode(input, scalars, radius);
    crate::filters::image::dilate_erode::image_dilate(&eroded, scalars, radius)
}

/// Morphological closing: dilation followed by erosion. Fills small dark holes.
pub fn image_morphological_closing(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let dilated = crate::filters::image::dilate_erode::image_dilate(input, scalars, radius);
    crate::filters::image::dilate_erode::image_erode(&dilated, scalars, radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_at_edge() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 100.0, 100.0],
                1,
            )));
        let result = image_morphological_gradient(&img, "v", 1);
        let arr = result.point_data().get_array("MorphGradient").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 50.0); // edge between 0 and 100
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0); // interior
    }

    #[test]
    fn uniform_zero_gradient() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));
        let result = image_morphological_gradient(&img, "v", 1);
        let arr = result.point_data().get_array("MorphGradient").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn opening_removes_noise() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 0.0, 0.0],
                1,
            )));
        let result = image_morphological_opening(&img, "v", 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0); // spike removed
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_morphological_gradient(&img, "nope", 1);
        assert!(r.point_data().get_array("MorphGradient").is_none());
    }
}
