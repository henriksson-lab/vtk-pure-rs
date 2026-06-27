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
    if n != nx * ny * dims[2] {
        return input.clone();
    }
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
        let slice = nx * ny;
        let iz = idx / slice;
        let rem = idx - iz * slice;
        let iy = rem / nx;
        let ix = rem % nx;
        let x_min = if ix == 0 { idx } else { idx - 1 };
        let x_max = if ix + 1 == nx { idx } else { idx + 1 };
        let y_min = if iy == 0 { idx } else { idx - nx };
        let y_max = if iy + 1 == ny { idx } else { idx + nx };
        let gx = if sp[0].abs() > 1e-15 {
            (vals[x_max] - vals[x_min]) * 0.5 / sp[0]
        } else {
            0.0
        };
        let gy = if sp[1].abs() > 1e-15 {
            (vals[y_max] - vals[y_min]) * 0.5 / sp[1]
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
    if n != nx * ny * dims[2] {
        return input.clone();
    }
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
            let slice = nx * ny;
            let iz = idx / slice;
            let rem = idx - iz * slice;
            let iy = rem / nx;
            let ix = rem % nx;
            let x_min = if ix == 0 { idx } else { idx - 1 };
            let x_max = if ix + 1 == nx { idx } else { idx + 1 };
            let y_min = if iy == 0 { idx } else { idx - nx };
            let y_max = if iy + 1 == ny { idx } else { idx + nx };
            let dfdx = if sp[0].abs() > 1e-15 {
                (vecs[x_max][0] - vecs[x_min][0]) * 0.5 / sp[0]
            } else {
                0.0
            };
            let dgdy = if sp[1].abs() > 1e-15 {
                (vecs[y_max][1] - vecs[y_min][1]) * 0.5 / sp[1]
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

    #[test]
    fn test_gvf_multiple_slices() {
        let img = ImageData::from_function(
            [4, 3, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| x + y + z * 10.0,
        );
        let g = gradient_vector_field(&img, "v");
        let arr = g.point_data().get_array("Gradient").unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(4 * 3 + 1 + 4, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        assert!((buf[1] - 1.0).abs() < 1e-10);
    }
}
