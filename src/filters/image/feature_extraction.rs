use crate::data::{AnyDataArray, DataArray, ImageData};

/// Detect Harris corners in a 2D ImageData.
///
/// Computes the Harris response R = det(M) - k*trace(M)² where M is the
/// structure tensor. High R = corner. Adds "HarrisResponse" array.
pub fn image_harris_corners(input: &ImageData, scalars: &str, k: f64, radius: usize) -> ImageData {
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
    let r = radius.max(1) as i64;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let get = |i: i64, j: i64, k: usize| -> f64 {
        values[k * ny * nx
            + (j.clamp(0, ny as i64 - 1) as usize) * nx
            + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    let mut response = vec![0.0f64; n];

    for kz in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;

                // Accumulate structure tensor
                let mut sxx = 0.0;
                let mut sxy = 0.0;
                let mut syy = 0.0;
                for dj in -r..=r {
                    for di in -r..=r {
                        let ni = ii + di;
                        let nj = jj + dj;
                        let gx = (get(ni + 1, nj, kz) - get(ni - 1, nj, kz)) / (2.0 * sp[0]);
                        let gy = (get(ni, nj + 1, kz) - get(ni, nj - 1, kz)) / (2.0 * sp[1]);
                        sxx += gx * gx;
                        sxy += gx * gy;
                        syy += gy * gy;
                    }
                }

                let det = sxx * syy - sxy * sxy;
                let trace = sxx + syy;
                response[kz * ny * nx + j * nx + i] = det - k * trace * trace;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HarrisResponse",
            response,
            1,
        )));
    img
}

/// Find local maxima of Harris response above a threshold.
/// Returns (row, col) pairs.
pub fn harris_corner_points(
    input: &ImageData,
    scalars: &str,
    k: f64,
    radius: usize,
    threshold: f64,
) -> Vec<(usize, usize)> {
    let result = image_harris_corners(input, scalars, k, radius);
    let arr = match result.point_data().get_array("HarrisResponse") {
        Some(a) => a,
        None => return vec![],
    };

    let dims = result.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;

    let mut buf = [0.0f64];
    let mut corners = Vec::new();

    for j in 1..ny - 1 {
        for i in 1..nx - 1 {
            arr.tuple_as_f64(j * nx + i, &mut buf);
            let v = buf[0];
            if v < threshold {
                continue;
            }

            // Check if local maximum
            let mut is_max = true;
            'check: for dj in -1i64..=1 {
                for di in -1i64..=1 {
                    if di == 0 && dj == 0 {
                        continue;
                    }
                    arr.tuple_as_f64(
                        (j as i64 + dj) as usize * nx + (i as i64 + di) as usize,
                        &mut buf,
                    );
                    if buf[0] >= v {
                        is_max = false;
                        break 'check;
                    }
                }
            }
            if is_max {
                corners.push((j, i));
            }
        }
    }

    corners
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harris_response() {
        let mut img = ImageData::with_dimensions(9, 9, 1);
        img.set_spacing([1.0; 3]);
        let mut values = vec![0.0; 81];
        // Create an L-shaped corner
        for j in 0..5 {
            for i in 0..5 {
                values[j * 9 + i] = 100.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_harris_corners(&img, "v", 0.04, 1);
        assert!(result.point_data().get_array("HarrisResponse").is_some());
    }

    #[test]
    fn uniform_no_corners() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0; 25],
                1,
            )));

        let corners = harris_corner_points(&img, "v", 0.04, 1, 1.0);
        assert!(corners.is_empty());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let r = image_harris_corners(&img, "nope", 0.04, 1);
        assert!(r.point_data().get_array("HarrisResponse").is_none());
    }
}
