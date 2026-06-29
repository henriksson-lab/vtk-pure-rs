use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Morphological top-hat transform: original minus opening.
///
/// The opening is erosion followed by dilation. The top-hat highlights
/// bright features smaller than the structuring element on a dark background.
/// Uses a cubic structuring element of the given `radius`.
pub fn top_hat(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let original = read_scalars(input, scalars);
    if original.is_empty() {
        return input.clone();
    }
    if radius == 0 {
        return write_scalars(input, scalars, vec![0.0; original.len()]);
    }
    let dims = input.dimensions();
    if dims[0] * dims[1] * dims[2] != original.len() {
        return input.clone();
    }
    let opened = dilate(&erode(&original, dims, radius), dims, radius);

    let result: Vec<f64> = original
        .iter()
        .zip(opened.iter())
        .map(|(&o, &op)| (o - op).max(0.0))
        .collect();

    write_scalars(input, scalars, result)
}

/// Morphological bottom-hat transform: closing minus original.
///
/// The closing is dilation followed by erosion. The bottom-hat highlights
/// dark features smaller than the structuring element on a bright background.
pub fn bottom_hat(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let original = read_scalars(input, scalars);
    if original.is_empty() {
        return input.clone();
    }
    if radius == 0 {
        return write_scalars(input, scalars, vec![0.0; original.len()]);
    }
    let dims = input.dimensions();
    if dims[0] * dims[1] * dims[2] != original.len() {
        return input.clone();
    }
    let closed = erode(&dilate(&original, dims, radius), dims, radius);

    let result: Vec<f64> = closed
        .iter()
        .zip(original.iter())
        .map(|(&c, &o)| (c - o).max(0.0))
        .collect();

    write_scalars(input, scalars, result)
}

fn read_scalars(input: &ImageData, scalars: &str) -> Vec<f64> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let n: usize = arr.num_tuples();
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }
    values
}

fn write_scalars(input: &ImageData, scalars: &str, result: Vec<f64>) -> ImageData {
    let mut img = input.clone();
    let mut new_attrs = DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

fn erode(values: &[f64], dims: [usize; 3], radius: usize) -> Vec<f64> {
    morphological_op(values, dims, radius, false)
}

fn dilate(values: &[f64], dims: [usize; 3], radius: usize) -> Vec<f64> {
    morphological_op(values, dims, radius, true)
}

fn morphological_op(values: &[f64], dims: [usize; 3], radius: usize, is_dilate: bool) -> Vec<f64> {
    let nx: usize = dims[0];
    let ny: usize = dims[1];
    let nz: usize = dims[2];
    let n: usize = nx * ny * nz;
    let r: i64 = radius as i64;
    let mut result: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut best: f64 = if is_dilate { f64::MIN } else { f64::MAX };

                for dk in -r..=r {
                    let kk: usize = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                    for dj in -r..=r {
                        let jj: usize = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                        for di in -r..=r {
                            let ii: usize = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                            let v: f64 = values[kk * ny * nx + jj * nx + ii];
                            if is_dilate {
                                best = best.max(v);
                            } else {
                                best = best.min(v);
                            }
                        }
                    }
                }

                result[k * ny * nx + j * nx + i] = best;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image_with_spike() -> ImageData {
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values = vec![0.0f64; 49];
        values[24] = 1.0; // center pixel bright spike
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", values, 1)));
        img
    }

    #[test]
    fn top_hat_detects_bright_spike() {
        let img = make_image_with_spike();
        let result = top_hat(&img, "data", 1);
        let arr = result.point_data().get_array("data").unwrap();
        let mut buf = [0.0f64];
        // The bright spike should be preserved (or partially) in top-hat
        arr.tuple_as_f64(24, &mut buf);
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn bottom_hat_on_bright_spike_is_zero() {
        let img = make_image_with_spike();
        let result = bottom_hat(&img, "data", 1);
        let arr = result.point_data().get_array("data").unwrap();
        // Bottom-hat = closing - original. For a bright spike, closing >= original,
        // but the spike region in the closing should equal the original spike,
        // so the result at the spike should be 0 or very small.
        let mut buf = [0.0f64];
        arr.tuple_as_f64(24, &mut buf);
        assert!(buf[0] < 1e-10);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = make_image_with_spike();
        let result = top_hat(&img, "nonexistent", 1);
        assert!(result.point_data().get_array("data").is_some());
    }
}
