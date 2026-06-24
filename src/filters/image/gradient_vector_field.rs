//! Compute gradient vector field from scalar images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute gradient vector field (2-component: dx, dy) using central differences.
pub fn gradient_vector_field(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let sp = input.spacing();

    let mut data = Vec::with_capacity(n * 2);
    for idx in 0..n {
        let iy = idx / nx;
        let ix = idx % nx;
        let gx = if ix > 0 && ix + 1 < nx {
            (vals[idx + 1] - vals[idx - 1]) / (2.0 * sp[0])
        } else if ix + 1 < nx {
            (vals[idx + 1] - vals[idx]) / sp[0]
        } else if ix > 0 {
            (vals[idx] - vals[idx - 1]) / sp[0]
        } else {
            0.0
        };
        let gy = if iy > 0 && iy + 1 < ny {
            (vals[idx + nx] - vals[idx - nx]) / (2.0 * sp[1])
        } else if iy + 1 < ny {
            (vals[idx + nx] - vals[idx]) / sp[1]
        } else if iy > 0 {
            (vals[idx] - vals[idx - nx]) / sp[1]
        } else {
            0.0
        };
        data.push(gx);
        data.push(gy);
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Gradient", data, 2)))
}

/// Compute gradient magnitude from gradient vector field.
pub fn gradient_magnitude_from_gvf(gvf: &ImageData) -> ImageData {
    let arr = match gvf.point_data().get_array("Gradient") {
        Some(a) if a.num_components() == 2 => a,
        _ => return gvf.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0] * buf[0] + buf[1] * buf[1]).sqrt()
        })
        .collect();
    let dims = gvf.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(gvf.spacing())
        .with_origin(gvf.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "GradMagnitude",
            data,
            1,
        )))
}

/// Compute divergence of a 2-component vector field.
pub fn divergence_2d(input: &ImageData, array_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 2 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let vecs: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1]]
        })
        .collect();
    let sp = input.spacing();

    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iy = idx / nx;
            let ix = idx % nx;
            let dfdx = if ix > 0 && ix + 1 < nx {
                (vecs[idx + 1][0] - vecs[idx - 1][0]) / (2.0 * sp[0])
            } else {
                0.0
            };
            let dgdy = if iy > 0 && iy + 1 < ny {
                (vecs[idx + nx][1] - vecs[idx - nx][1]) / (2.0 * sp[1])
            } else {
                0.0
            };
            dfdx + dgdy
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "Divergence",
            data,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gvf() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x * x + y,
        );
        let g = gradient_vector_field(&img, "v");
        let arr = g.point_data().get_array("Gradient").unwrap();
        assert_eq!(arr.num_components(), 2);
    }
    #[test]
    fn test_mag() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let g = gradient_vector_field(&img, "v");
        let m = gradient_magnitude_from_gvf(&g);
        assert!(m.point_data().get_array("GradMagnitude").is_some());
    }
    #[test]
    fn test_div() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let g = gradient_vector_field(&img, "v");
        let d = divergence_2d(&g, "Gradient");
        assert!(d.point_data().get_array("Divergence").is_some());
    }
}
