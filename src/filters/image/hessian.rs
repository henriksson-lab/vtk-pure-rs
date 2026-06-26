use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the Hessian determinant of a scalar field on ImageData.
///
/// The Hessian determinant det(H) = fxx*fyy - fxy^2 (2D) is useful for
/// blob detection (positive = bright blob, negative = saddle point).
/// Adds "HessianDet" scalar array.
pub fn image_hessian_det(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let sp = input.spacing();
    if arr.num_tuples() != n {
        return input.clone();
    }

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let get = |i: i64, j: i64, k: i64| -> f64 {
        let ii = i.clamp(0, nx as i64 - 1) as usize;
        let jj = j.clamp(0, ny as i64 - 1) as usize;
        let kk = k.clamp(0, nz as i64 - 1) as usize;
        values[kk * ny * nx + jj * nx + ii]
    };

    let mut det = vec![0.0f64; n];
    let hx2 = sp[0] * sp[0];
    let hy2 = sp[1] * sp[1];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;
                let kk = k as i64;
                let c = get(ii, jj, kk);
                let fxx = (get(ii + 1, jj, kk) - 2.0 * c + get(ii - 1, jj, kk)) / hx2;
                let fyy = (get(ii, jj + 1, kk) - 2.0 * c + get(ii, jj - 1, kk)) / hy2;
                let fxy =
                    (get(ii + 1, jj + 1, kk) - get(ii - 1, jj + 1, kk) - get(ii + 1, jj - 1, kk)
                        + get(ii - 1, jj - 1, kk))
                        / (4.0 * sp[0] * sp[1]);
                det[k * ny * nx + j * nx + i] = fxx * fyy - fxy * fxy;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("HessianDet", det, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paraboloid_positive() {
        // f = x² + y²: Hessian det = fxx*fyy - fxy² = 2*2 - 0 = 4
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let mut values = Vec::new();
        for j in 0..5 {
            for i in 0..5 {
                let x = i as f64 - 2.0;
                let y = j as f64 - 2.0;
                values.push(x * x + y * y);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", values, 1)));

        let result = image_hessian_det(&img, "f");
        let arr = result.point_data().get_array("HessianDet").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf); // center
        assert!((buf[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn saddle_negative() {
        // f = x² - y²: Hessian det = 2*(-2) - 0 = -4
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let mut values = Vec::new();
        for j in 0..5 {
            for i in 0..5 {
                let x = i as f64 - 2.0;
                let y = j as f64 - 2.0;
                values.push(x * x - y * y);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", values, 1)));

        let result = image_hessian_det(&img, "f");
        let arr = result.point_data().get_array("HessianDet").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert!((buf[0] + 4.0).abs() < 1e-10); // -4 for saddle
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_hessian_det(&img, "nope");
        assert!(result.point_data().get_array("HessianDet").is_none());
    }
}
