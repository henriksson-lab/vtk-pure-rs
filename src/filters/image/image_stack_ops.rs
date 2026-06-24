//! Operations on stacks of 2D images (3D volumes).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Maximum intensity projection along Z axis.
pub fn max_intensity_projection(input: &ImageData, scalars: &str) -> ImageData {
    project_along_z(input, scalars, "MIP", |vals| {
        vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    })
}

/// Minimum intensity projection along Z axis.
pub fn min_intensity_projection(input: &ImageData, scalars: &str) -> ImageData {
    project_along_z(input, scalars, "MinIP", |vals| {
        vals.iter().cloned().fold(f64::INFINITY, f64::min)
    })
}

/// Average intensity projection along Z axis.
pub fn mean_intensity_projection(input: &ImageData, scalars: &str) -> ImageData {
    project_along_z(input, scalars, "MeanIP", |vals| {
        if vals.is_empty() {
            0.0
        } else {
            vals.iter().sum::<f64>() / vals.len() as f64
        }
    })
}

/// Sum intensity projection along Z axis.
pub fn sum_intensity_projection(input: &ImageData, scalars: &str) -> ImageData {
    project_along_z(input, scalars, "SumIP", |vals| vals.iter().sum())
}

fn project_along_z(
    input: &ImageData,
    scalars: &str,
    out_name: &str,
    reduce: impl Fn(&[f64]) -> f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut data = Vec::with_capacity(nx * ny);
    for iy in 0..ny {
        for ix in 0..nx {
            let col: Vec<f64> = (0..nz)
                .map(|iz| vals[ix + iy * nx + iz * nx * ny])
                .collect();
            data.push(reduce(&col));
        }
    }

    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing([input.spacing()[0], input.spacing()[1], 1.0])
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(out_name, data, 1)))
}

/// Extract a single Z slice from a 3D volume.
pub fn extract_z_slice(input: &ImageData, scalars: &str, z_index: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let offset = z_index * nx * ny;
    let data: Vec<f64> = (0..nx * ny)
        .map(|i| {
            let idx = offset + i;
            if idx < arr.num_tuples() {
                arr.tuple_as_f64(idx, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();

    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing([input.spacing()[0], input.spacing()[1], 1.0])
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mip() {
        let img = ImageData::from_function(
            [4, 4, 3],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, z| z,
        );
        let r = max_intensity_projection(&img, "v");
        assert_eq!(r.dimensions(), [4, 4, 1]);
        let arr = r.point_data().get_array("MIP").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_slice() {
        let img = ImageData::from_function(
            [4, 4, 3],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, z| z * 10.0,
        );
        let r = extract_z_slice(&img, "v", 1);
        assert_eq!(r.dimensions(), [4, 4, 1]);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 10.0).abs() < 1e-10);
    }
}
