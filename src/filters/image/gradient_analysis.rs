//! Gradient analysis on ImageData: magnitude, direction, divergence, curl.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute gradient magnitude + direction of a scalar field.
pub fn gradient_analysis(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut gx = vec![0.0f64; n];
    let mut gy = vec![0.0f64; n];
    let mut gz = vec![0.0f64; n];
    let mut mag = vec![0.0f64; n];

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let xm = if ix > 0 { vals[idx - 1] } else { vals[idx] };
                let xp = if ix + 1 < dims[0] {
                    vals[idx + 1]
                } else {
                    vals[idx]
                };
                let ym = if iy > 0 {
                    vals[idx - dims[0]]
                } else {
                    vals[idx]
                };
                let yp = if iy + 1 < dims[1] {
                    vals[idx + dims[0]]
                } else {
                    vals[idx]
                };
                let zm = if iz > 0 {
                    vals[idx - dims[0] * dims[1]]
                } else {
                    vals[idx]
                };
                let zp = if iz + 1 < dims[2] {
                    vals[idx + dims[0] * dims[1]]
                } else {
                    vals[idx]
                };
                let dx = (ix > 0 && ix + 1 < dims[0]) as usize + 1;
                let dy = (iy > 0 && iy + 1 < dims[1]) as usize + 1;
                let dz = (iz > 0 && iz + 1 < dims[2]) as usize + 1;
                gx[idx] = (xp - xm) / (dx as f64 * sp[0]);
                gy[idx] = (yp - ym) / (dy as f64 * sp[1]);
                gz[idx] = (zp - zm) / (dz as f64 * sp[2]);
                mag[idx] = (gx[idx].powi(2) + gy[idx].powi(2) + gz[idx].powi(2)).sqrt();
            }
        }
    }

    let mut grad_vec = Vec::with_capacity(n * 3);
    for i in 0..n {
        grad_vec.push(gx[i]);
        grad_vec.push(gy[i]);
        grad_vec.push(gz[i]);
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientVector",
            grad_vec,
            3,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GradientMagnitude",
            mag,
            1,
        )));
    result
}

/// Compute divergence of a vector field on ImageData.
pub fn divergence(image: &ImageData, vector_array: &str) -> ImageData {
    let arr = match image.point_data().get_array(vector_array) {
        Some(a) if a.num_components() == 3 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64; 3];
    let vx: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let vy: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[1]
        })
        .collect();
    let vz: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[2]
        })
        .collect();

    let mut div = vec![0.0f64; n];
    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let dvx = if ix > 0 && ix + 1 < dims[0] {
                    (vx[idx + 1] - vx[idx - 1]) / (2.0 * sp[0])
                } else {
                    0.0
                };
                let dvy = if iy > 0 && iy + 1 < dims[1] {
                    (vy[idx + dims[0]] - vy[idx - dims[0]]) / (2.0 * sp[1])
                } else {
                    0.0
                };
                let dvz = if iz > 0 && iz + 1 < dims[2] {
                    (vz[idx + dims[0] * dims[1]] - vz[idx - dims[0] * dims[1]]) / (2.0 * sp[2])
                } else {
                    0.0
                };
                div[idx] = dvx + dvy + dvz;
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Divergence", div, 1)));
    result
}

/// Compute curl magnitude of a vector field on ImageData.
pub fn curl_magnitude(image: &ImageData, vector_array: &str) -> ImageData {
    let arr = match image.point_data().get_array(vector_array) {
        Some(a) if a.num_components() == 3 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let n = dims[0] * dims[1] * dims[2];
    let mut buf = [0.0f64; 3];
    let vals: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1], buf[2]]
        })
        .collect();

    let mut curl_mag = vec![0.0f64; n];
    for iz in 1..dims[2].saturating_sub(1) {
        for iy in 1..dims[1].saturating_sub(1) {
            for ix in 1..dims[0].saturating_sub(1) {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                let dwdy = (vals[idx + dims[0]][2] - vals[idx - dims[0]][2]) / (2.0 * sp[1]);
                let dvdz = (vals[idx + dims[0] * dims[1]][1] - vals[idx - dims[0] * dims[1]][1])
                    / (2.0 * sp[2]);
                let dudz = (vals[idx + dims[0] * dims[1]][0] - vals[idx - dims[0] * dims[1]][0])
                    / (2.0 * sp[2]);
                let dwdx = (vals[idx + 1][2] - vals[idx - 1][2]) / (2.0 * sp[0]);
                let dvdx = (vals[idx + 1][1] - vals[idx - 1][1]) / (2.0 * sp[0]);
                let dudy = (vals[idx + dims[0]][0] - vals[idx - dims[0]][0]) / (2.0 * sp[1]);
                let cx = dwdy - dvdz;
                let cy = dudz - dwdx;
                let cz = dvdx - dudy;
                curl_mag[idx] = (cx * cx + cy * cy + cz * cz).sqrt();
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurlMagnitude",
            curl_mag,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gradient() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "f",
            |x, _, _| x,
        );
        let result = gradient_analysis(&img, "f");
        assert!(result.point_data().get_array("GradientVector").is_some());
        assert!(result.point_data().get_array("GradientMagnitude").is_some());
    }
    #[test]
    fn div() {
        let mut img = ImageData::with_dimensions(5, 5, 5).with_spacing([1.0, 1.0, 1.0]);
        let n = 125;
        let v: Vec<f64> = (0..n)
            .flat_map(|i| {
                let x = i % 5;
                vec![x as f64, 0.0, 0.0]
            })
            .collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("vel", v, 3)));
        let result = divergence(&img, "vel");
        assert!(result.point_data().get_array("Divergence").is_some());
    }
}
