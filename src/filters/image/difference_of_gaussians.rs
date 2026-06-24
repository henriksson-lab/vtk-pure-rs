use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Difference of Gaussians (DoG) on ImageData.
///
/// DoG = Gaussian(sigma1) - Gaussian(sigma2), approximates Laplacian of Gaussian.
/// Useful for blob detection (positive = bright blob, negative = dark blob).
pub fn image_dog(
    input: &ImageData,
    scalars: &str,
    sigma1: f64,
    sigma2: f64,
    radius: usize,
) -> ImageData {
    let g1 = crate::filters::image::gaussian_smooth::image_gaussian_smooth(
        input, scalars, sigma1, radius,
    );
    let g2 = crate::filters::image::gaussian_smooth::image_gaussian_smooth(
        input, scalars, sigma2, radius,
    );

    let a1 = match g1.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let a2 = match g2.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = a1.num_tuples().min(a2.num_tuples());
    let mut b1 = [0.0f64];
    let mut b2 = [0.0f64];
    let dog: Vec<f64> = (0..n)
        .map(|i| {
            a1.tuple_as_f64(i, &mut b1);
            a2.tuple_as_f64(i, &mut b2);
            b1[0] - b2[0]
        })
        .collect();

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DoG", dog, 1)));
    img
}

/// Detect blobs using DoG zero-crossings. Returns (row,col) positions.
pub fn detect_blobs(
    input: &ImageData,
    scalars: &str,
    sigma1: f64,
    sigma2: f64,
    radius: usize,
    threshold: f64,
) -> Vec<(usize, usize)> {
    let result = image_dog(input, scalars, sigma1, sigma2, radius);
    let arr = match result.point_data().get_array("DoG") {
        Some(a) => a,
        None => return vec![],
    };

    let dims = result.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let mut buf = [0.0f64];

    let mut blobs = Vec::new();
    for j in 1..ny.saturating_sub(1) {
        for i in 1..nx.saturating_sub(1) {
            arr.tuple_as_f64(j * nx + i, &mut buf);
            if buf[0].abs() < threshold {
                continue;
            }
            // Local maximum check
            let v = buf[0];
            let mut is_extremum = true;
            for dj in -1i64..=1 {
                for di in -1i64..=1 {
                    if di == 0 && dj == 0 {
                        continue;
                    }
                    arr.tuple_as_f64(
                        (j as i64 + dj) as usize * nx + (i as i64 + di) as usize,
                        &mut buf,
                    );
                    if buf[0].abs() >= v.abs() && (di != 0 || dj != 0) {
                        is_extremum = false;
                        break;
                    }
                }
                if !is_extremum {
                    break;
                }
            }
            if is_extremum {
                blobs.push((j, i));
            }
        }
    }
    blobs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dog_basic() {
        let mut img = ImageData::with_dimensions(9, 9, 1);
        let mut values = vec![0.0; 81];
        values[40] = 100.0; // bright blob at center
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_dog(&img, "v", 0.5, 2.0, 2);
        assert!(result.point_data().get_array("DoG").is_some());
    }

    #[test]
    fn uniform_zero_dog() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0; 25],
                1,
            )));

        let result = image_dog(&img, "v", 0.5, 2.0, 1);
        let arr = result.point_data().get_array("DoG").unwrap();
        let mut buf = [0.0f64];
        for i in 0..25 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let r = image_dog(&img, "nope", 0.5, 2.0, 1);
        assert!(r.point_data().get_array("DoG").is_none());
    }
}
