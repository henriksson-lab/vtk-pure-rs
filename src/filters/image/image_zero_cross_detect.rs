//! Zero crossing indicator
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_zero_cross_detect(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    if n == 0 || arr.num_tuples() < n {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let idx = |i: usize, j: usize, k: usize| -> usize { k * ny * nx + j * nx + i };
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let k = i / (nx * ny);
            let j = (i / nx) % ny;
            let x = i % nx;
            let v = values[i];
            let crosses_zero = |a: f64, b: f64| (a <= 0.0 && b > 0.0) || (a >= 0.0 && b < 0.0);
            let mut crosses = false;
            if x > 0 {
                crosses |= crosses_zero(v, values[idx(x - 1, j, k)]);
            }
            if x + 1 < nx {
                crosses |= crosses_zero(v, values[idx(x + 1, j, k)]);
            }
            if j > 0 {
                crosses |= crosses_zero(v, values[idx(x, j - 1, k)]);
            }
            if j + 1 < ny {
                crosses |= crosses_zero(v, values[idx(x, j + 1, k)]);
            }
            if k > 0 {
                crosses |= crosses_zero(v, values[idx(x, j, k - 1)]);
            }
            if k + 1 < nz {
                crosses |= crosses_zero(v, values[idx(x, j, k + 1)]);
            }
            if crosses {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut output = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)));
    output.set_extent(input.extent());
    output
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_zero_cross_detect(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn detects_neighbor_sign_changes() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![-1.0, 1.0, 2.0],
                1,
            )));

        let r = image_zero_cross_detect(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut value = [0.0f64];
        arr.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 1.0);
        arr.tuple_as_f64(1, &mut value);
        assert_eq!(value[0], 1.0);
        arr.tuple_as_f64(2, &mut value);
        assert_eq!(value[0], 0.0);
    }
}
