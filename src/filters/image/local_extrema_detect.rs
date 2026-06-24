use crate::data::{AnyDataArray, DataArray, ImageData};

/// Detect local maxima in an ImageData scalar field using 26-connectivity.
///
/// A voxel is a local maximum if its value is strictly greater than all 26
/// neighbors. Adds a "LocalMaxima" binary array (1.0 or 0.0) to point data.
pub fn detect_local_maxima(input: &ImageData, scalars: &str) -> ImageData {
    detect_extrema(input, scalars, true)
}

/// Detect local minima in an ImageData scalar field using 26-connectivity.
///
/// A voxel is a local minimum if its value is strictly less than all 26
/// neighbors. Adds a "LocalMinima" binary array (1.0 or 0.0) to point data.
pub fn detect_local_minima(input: &ImageData, scalars: &str) -> ImageData {
    detect_extrema(input, scalars, false)
}

fn detect_extrema(input: &ImageData, scalars: &str, find_maxima: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n: usize = nx * ny * nz;

    // Read all scalar values
    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };

    let mut result = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let v: f64 = values[idx(i, j, k)];
                let mut is_extremum: bool = true;

                // Check all 26 neighbors
                'outer: for dk in -1i64..=1 {
                    for dj in -1i64..=1 {
                        for di in -1i64..=1 {
                            if di == 0 && dj == 0 && dk == 0 {
                                continue;
                            }
                            let ni: i64 = i as i64 + di;
                            let nj: i64 = j as i64 + dj;
                            let nk: i64 = k as i64 + dk;
                            if ni < 0 || nj < 0 || nk < 0 {
                                continue;
                            }
                            let ni = ni as usize;
                            let nj = nj as usize;
                            let nk = nk as usize;
                            if ni >= nx || nj >= ny || nk >= nz {
                                continue;
                            }
                            let nv: f64 = values[idx(ni, nj, nk)];
                            if find_maxima {
                                if nv >= v {
                                    is_extremum = false;
                                    break 'outer;
                                }
                            } else {
                                if nv <= v {
                                    is_extremum = false;
                                    break 'outer;
                                }
                            }
                        }
                    }
                }

                result[idx(i, j, k)] = if is_extremum { 1.0 } else { 0.0 };
            }
        }
    }

    let name: &str = if find_maxima {
        "LocalMaxima"
    } else {
        "LocalMinima"
    };
    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, result, 1)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_peak_26_connectivity() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n: usize = 5 * 5 * 5;
        let mut values = vec![0.0f64; n];
        // Place peak at center (2,2,2) = index 2*25 + 2*5 + 2 = 62
        values[62] = 10.0;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalar", values, 1)));

        let result = detect_local_maxima(&img, "scalar");
        let arr = result.point_data().get_array("LocalMaxima").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(62, &mut buf);
        assert_eq!(buf[0], 1.0); // center is max

        // A diagonal neighbor at (1,1,1) = index 1*25+1*5+1 = 31 should not be a max
        arr.tuple_as_f64(31, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn single_valley_26_connectivity() {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n: usize = 5 * 5 * 5;
        let mut values = vec![10.0f64; n];
        values[62] = 0.0; // valley at center
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalar", values, 1)));

        let result = detect_local_minima(&img, "scalar");
        let arr = result.point_data().get_array("LocalMinima").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(62, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = detect_local_maxima(&img, "nonexistent");
        assert!(result.point_data().get_array("LocalMaxima").is_none());
    }
}
