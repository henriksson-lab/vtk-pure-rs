use crate::data::{AnyDataArray, DataArray, ImageData};

/// Find local maxima in an ImageData scalar field.
///
/// A voxel is a local maximum if its value is strictly greater than
/// all 6-connected neighbors. Adds a "LocalMaxima" binary array.
pub fn image_local_maxima(input: &ImageData, scalars: &str) -> ImageData {
    image_extrema(input, scalars, true)
}

/// Find local minima in an ImageData scalar field.
pub fn image_local_minima(input: &ImageData, scalars: &str) -> ImageData {
    image_extrema(input, scalars, false)
}

fn image_extrema(input: &ImageData, scalars: &str, find_maxima: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut values = vec![0.0f64; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    let mut result = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let v = values[idx(i, j, k)];
                let mut is_extremum = true;
                let neighbors = [
                    (i.wrapping_sub(1), j, k),
                    (i + 1, j, k),
                    (i, j.wrapping_sub(1), k),
                    (i, j + 1, k),
                    (i, j, k.wrapping_sub(1)),
                    (i, j, k + 1),
                ];
                for &(ni, nj, nk) in &neighbors {
                    if ni < nx && nj < ny && nk < nz {
                        let nv = values[idx(ni, nj, nk)];
                        if find_maxima {
                            if nv >= v {
                                is_extremum = false;
                                break;
                            }
                        } else {
                            if nv <= v {
                                is_extremum = false;
                                break;
                            }
                        }
                    }
                }
                result[idx(i, j, k)] = if is_extremum { 1.0 } else { 0.0 };
            }
        }
    }

    let name = if find_maxima {
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
    fn single_peak() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0; 25];
        values[12] = 10.0; // center peak
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_local_maxima(&img, "v");
        let arr = result.point_data().get_array("LocalMaxima").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0); // center is max
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // corner not max (neighbors also 0)
    }

    #[test]
    fn single_valley() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![10.0; 25];
        values[12] = 0.0; // center valley
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_local_minima(&img, "v");
        let arr = result.point_data().get_array("LocalMinima").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = image_local_maxima(&img, "nope");
        assert!(result.point_data().get_array("LocalMaxima").is_none());
    }
}
