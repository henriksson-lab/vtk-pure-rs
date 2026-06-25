use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute the curl (rotation) of a 3D vector field on ImageData.
///
/// Takes the name of a 3-component point data array and produces a new 3-component
/// "Curl" array using central differences:
///
///   curl_x = dVz/dy - dVy/dz
///   curl_y = dVx/dz - dVz/dx
///   curl_z = dVy/dx - dVx/dy
pub fn compute_curl(input: &ImageData, vector_array: &str) -> ImageData {
    let arr = match input.point_data().get_array(vector_array) {
        Some(a) => a,
        None => return input.clone(),
    };

    if arr.num_components() != 3 {
        return input.clone();
    }

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let spacing = input.spacing();
    let n: usize = nx * ny * nz;

    // Read vector field into flat arrays for each component.
    let mut vx: Vec<f64> = vec![0.0; n];
    let mut vy: Vec<f64> = vec![0.0; n];
    let mut vz: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64; 3];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vx[i] = buf[0];
        vy[i] = buf[1];
        vz[i] = buf[2];
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut curl_data: Vec<f64> = vec![0.0; n * 3];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pi: usize = idx(i, j, k);

                // Central differences with clamped boundaries.
                let im: usize = if i > 0 { i - 1 } else { 0 };
                let ip: usize = if i + 1 < nx { i + 1 } else { nx - 1 };
                let jm: usize = if j > 0 { j - 1 } else { 0 };
                let jp: usize = if j + 1 < ny { j + 1 } else { ny - 1 };
                let km: usize = if k > 0 { k - 1 } else { 0 };
                let kp: usize = if k + 1 < nz { k + 1 } else { nz - 1 };

                let dx_span: f64 = (ip - im) as f64 * spacing[0];
                let dy_span: f64 = (jp - jm) as f64 * spacing[1];
                let dz_span: f64 = (kp - km) as f64 * spacing[2];

                // dVz/dy
                let dvz_dy: f64 = if dy_span > 1e-15 {
                    (vz[idx(i, jp, k)] - vz[idx(i, jm, k)]) / dy_span
                } else {
                    0.0
                };
                // dVy/dz
                let dvy_dz: f64 = if dz_span > 1e-15 {
                    (vy[idx(i, j, kp)] - vy[idx(i, j, km)]) / dz_span
                } else {
                    0.0
                };
                // dVx/dz
                let dvx_dz: f64 = if dz_span > 1e-15 {
                    (vx[idx(i, j, kp)] - vx[idx(i, j, km)]) / dz_span
                } else {
                    0.0
                };
                // dVz/dx
                let dvz_dx: f64 = if dx_span > 1e-15 {
                    (vz[idx(ip, j, k)] - vz[idx(im, j, k)]) / dx_span
                } else {
                    0.0
                };
                // dVy/dx
                let dvy_dx: f64 = if dx_span > 1e-15 {
                    (vy[idx(ip, j, k)] - vy[idx(im, j, k)]) / dx_span
                } else {
                    0.0
                };
                // dVx/dy
                let dvx_dy: f64 = if dy_span > 1e-15 {
                    (vx[idx(i, jp, k)] - vx[idx(i, jm, k)]) / dy_span
                } else {
                    0.0
                };

                curl_data[pi * 3] = dvz_dy - dvy_dz;
                curl_data[pi * 3 + 1] = dvx_dz - dvz_dx;
                curl_data[pi * 3 + 2] = dvy_dx - dvx_dy;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Curl", curl_data, 3)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_field_has_zero_curl() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.set_spacing([1.0, 1.0, 1.0]);
        let n: usize = 27;
        let mut data: Vec<f64> = Vec::with_capacity(n * 3);
        for _ in 0..n {
            data.push(1.0);
            data.push(2.0);
            data.push(3.0);
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("V", data, 3)));
        let result = compute_curl(&img, "V");
        let curl = result.point_data().get_array("Curl").unwrap();
        let mut buf = [0.0f64; 3];
        // Interior point (1,1,1) = index 13
        curl.tuple_as_f64(13, &mut buf);
        assert!(buf[0].abs() < 1e-10);
        assert!(buf[1].abs() < 1e-10);
        assert!(buf[2].abs() < 1e-10);
    }

    #[test]
    fn curl_of_rotation_field() {
        // V = (-y, x, 0) should have curl = (0, 0, 2).
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        let nx: usize = 5;
        let ny: usize = 5;
        let n: usize = nx * ny;
        let mut data: Vec<f64> = Vec::with_capacity(n * 3);
        for j in 0..ny {
            for i in 0..nx {
                let x: f64 = i as f64;
                let y: f64 = j as f64;
                data.push(-y); // Vx = -y
                data.push(x); // Vy = x
                data.push(0.0);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("V", data, 3)));
        let result = compute_curl(&img, "V");
        let curl = result.point_data().get_array("Curl").unwrap();
        let mut buf = [0.0f64; 3];
        // Interior point (2,2,0) = index 12
        curl.tuple_as_f64(12, &mut buf);
        assert!(buf[0].abs() < 1e-10, "curl_x = {}", buf[0]);
        assert!(buf[1].abs() < 1e-10, "curl_y = {}", buf[1]);
        assert!((buf[2] - 2.0).abs() < 1e-10, "curl_z = {}", buf[2]);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = compute_curl(&img, "nonexistent");
        assert_eq!(result.dimensions(), img.dimensions());
    }
}
