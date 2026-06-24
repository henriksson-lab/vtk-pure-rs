use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the divergence of the gradient field on ImageData.
///
/// This is equivalent to the Laplacian but computed as div(grad(f)).
/// Adds a "Divergence" scalar array.
pub fn image_divergence(input: &ImageData, scalars: &str) -> ImageData {
    // Divergence of gradient = Laplacian, reuse existing impl
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let sp = input.spacing();

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

    let mut div = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;
                let kk = k as i64;
                let c = get(ii, jj, kk);
                let dxx = (get(ii + 1, jj, kk) - 2.0 * c + get(ii - 1, jj, kk)) / (sp[0] * sp[0]);
                let dyy = (get(ii, jj + 1, kk) - 2.0 * c + get(ii, jj - 1, kk)) / (sp[1] * sp[1]);
                let dzz = (get(ii, jj, kk + 1) - 2.0 * c + get(ii, jj, kk - 1)) / (sp[2] * sp[2]);
                div[k * ny * nx + j * nx + i] = dxx + dyy + dzz;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Divergence", div, 1)));
    img
}

/// Compute curl magnitude of a vector field stored as 3 scalar arrays.
///
/// Given arrays for vx, vy, vz components, computes |curl(v)|.
/// Adds "CurlMagnitude" scalar array.
pub fn image_curl_magnitude(
    input: &ImageData,
    vx_name: &str,
    vy_name: &str,
    vz_name: &str,
) -> ImageData {
    let vx = match input.point_data().get_array(vx_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let vy = match input.point_data().get_array(vy_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let vz = match input.point_data().get_array(vz_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let sp = input.spacing();

    let mut vx_v = vec![0.0f64; n];
    let mut vy_v = vec![0.0f64; n];
    let mut vz_v = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        vx.tuple_as_f64(i, &mut buf);
        vx_v[i] = buf[0];
    }
    for i in 0..n {
        vy.tuple_as_f64(i, &mut buf);
        vy_v[i] = buf[0];
    }
    for i in 0..n {
        vz.tuple_as_f64(i, &mut buf);
        vz_v[i] = buf[0];
    }

    let get = |v: &[f64], i: i64, j: i64, k: i64| -> f64 {
        v[(k.clamp(0, nz as i64 - 1) as usize) * ny * nx
            + (j.clamp(0, ny as i64 - 1) as usize) * nx
            + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    let mut curl = vec![0.0f64; n];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;
                let kk = k as i64;
                let dvz_dy =
                    (get(&vz_v, ii, jj + 1, kk) - get(&vz_v, ii, jj - 1, kk)) / (2.0 * sp[1]);
                let dvy_dz =
                    (get(&vy_v, ii, jj, kk + 1) - get(&vy_v, ii, jj, kk - 1)) / (2.0 * sp[2]);
                let dvx_dz =
                    (get(&vx_v, ii, jj, kk + 1) - get(&vx_v, ii, jj, kk - 1)) / (2.0 * sp[2]);
                let dvz_dx =
                    (get(&vz_v, ii + 1, jj, kk) - get(&vz_v, ii - 1, jj, kk)) / (2.0 * sp[0]);
                let dvy_dx =
                    (get(&vy_v, ii + 1, jj, kk) - get(&vy_v, ii - 1, jj, kk)) / (2.0 * sp[0]);
                let dvx_dy =
                    (get(&vx_v, ii, jj + 1, kk) - get(&vx_v, ii, jj - 1, kk)) / (2.0 * sp[1]);
                let cx = dvz_dy - dvy_dz;
                let cy = dvx_dz - dvz_dx;
                let cz = dvy_dx - dvx_dy;
                curl[k * ny * nx + j * nx + i] = (cx * cx + cy * cy + cz * cz).sqrt();
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurlMagnitude",
            curl,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divergence_constant() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![5.0; 27],
                1,
            )));
        let result = image_divergence(&img, "f");
        let arr = result.point_data().get_array("Divergence").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(13, &mut buf);
        assert!(buf[0].abs() < 1e-10);
    }

    #[test]
    fn curl_zero_for_gradient() {
        // v = grad(x^2) = (2x, 0, 0) -> curl = 0
        let mut img = ImageData::with_dimensions(5, 5, 5);
        img.set_spacing([1.0; 3]);
        let n = 125;
        let mut vx = vec![0.0; n];
        let vy = vec![0.0; n];
        let vz = vec![0.0; n];
        for k in 0..5 {
            for j in 0..5 {
                for i in 0..5 {
                    vx[k * 25 + j * 5 + i] = 2.0 * i as f64;
                }
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vx", vx, 1)));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vy", vy, 1)));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vz", vz, 1)));

        let result = image_curl_magnitude(&img, "vx", "vy", "vz");
        let arr = result.point_data().get_array("CurlMagnitude").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(62, &mut buf); // center
        assert!(buf[0] < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let r = image_divergence(&img, "nope");
        assert!(r.point_data().get_array("Divergence").is_none());
    }
}
