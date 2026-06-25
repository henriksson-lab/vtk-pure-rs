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
    let num_components = arr.num_components();
    if n == 0 || num_components == 0 {
        return input.clone();
    }
    let r = radius as i64;

    let mut buf = vec![0.0f64; num_components];
    let mut values = vec![0.0f64; n * num_components];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let offset = i * num_components;
        values[offset..offset + num_components].copy_from_slice(&buf);
    }

    let mut result = vec![0.0f64; n * num_components];
    let radius = r as f64 + 0.5;
    let radius2 = radius * radius;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let out_idx = k * ny * nx + j * nx + i;
                for component in 0..num_components {
                    let mut lo = f64::INFINITY;
                    let mut hi = f64::NEG_INFINITY;
                    for dk in -r..=r {
                        let kk = k as i64 + dk;
                        if kk < 0 || kk >= nz as i64 {
                            continue;
                        }
                        for dj in -r..=r {
                            let jj = j as i64 + dj;
                            if jj < 0 || jj >= ny as i64 {
                                continue;
                            }
                            for di in -r..=r {
                                let ii = i as i64 + di;
                                if ii < 0 || ii >= nx as i64 {
                                    continue;
                                }
                                let d2 =
                                    (di as f64).powi(2) + (dj as f64).powi(2) + (dk as f64).powi(2);
                                if d2 > radius2 {
                                    continue;
                                }
                                let v = values[(kk as usize * ny * nx
                                    + jj as usize * nx
                                    + ii as usize)
                                    * num_components
                                    + component];
                                lo = lo.min(v);
                                hi = hi.max(v);
                            }
                        }
                    }
                    result[out_idx * num_components + component] = hi - lo;
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MorphGradient",
            result,
            num_components,
        )));
    img
}

/// Morphological opening: erosion followed by dilation. Removes small bright features.
pub fn image_morphological_opening(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let eroded = crate::filters::core::image::dilate_erode::image_erode(input, scalars, radius);
    crate::filters::core::image::dilate_erode::image_dilate(&eroded, scalars, radius)
}

/// Morphological closing: dilation followed by erosion. Fills small dark holes.
pub fn image_morphological_closing(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let dilated = crate::filters::core::image::dilate_erode::image_dilate(input, scalars, radius);
    crate::filters::core::image::dilate_erode::image_erode(&dilated, scalars, radius)
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

    #[test]
    fn processes_all_components_independently() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![
                    0.0, 10.0, //
                    5.0, 10.0, //
                    5.0, 20.0,
                ],
                2,
            )));

        let result = image_morphological_gradient(&img, "v", 1);
        let arr = result.point_data().get_array("MorphGradient").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [5.0, 10.0]);
    }

    #[test]
    fn zero_radius_returns_zero_gradient() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 3.0],
                1,
            )));

        let result = image_morphological_gradient(&img, "v", 0);
        let arr = result.point_data().get_array("MorphGradient").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
