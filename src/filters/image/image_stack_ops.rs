//! Operations on stacks of 2D images (3D volumes).

use crate::data::{AnyDataArray, DataArray, ImageData};

pub use crate::filters::image::slice_ops::extract_z_slice;

/// Maximum intensity projection along Z axis.
///
/// Thin wrapper over the single implementation in
/// [`crate::filters::image::slice_extract::max_intensity_projection`] (axis 2),
/// keeping this module's `"MIP"` output-array naming convention that matches
/// its `MinIP`/`MeanIP`/`SumIP` siblings.
pub fn max_intensity_projection(input: &ImageData, scalars: &str) -> ImageData {
    match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => {}
        _ => return input.clone(),
    }
    let mut out = crate::filters::image::slice_extract::max_intensity_projection(input, scalars, 2);
    if let Some(mut arr) = out.point_data_mut().remove_array(scalars) {
        arr.set_name("MIP");
        out.point_data_mut().add_array(arr);
    }
    out
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

    let spacing = input.spacing();
    let mut origin = input.origin();
    if nz > 0 {
        origin[2] += 0.5 * spacing[2] * (nz - 1) as f64;
    }

    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing(spacing)
        .with_origin(origin)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(out_name, data, 1)))
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
        assert_eq!(r.spacing(), [1.0, 1.0, 1.0]);
        assert_eq!(r.origin(), [0.0, 0.0, 1.0]);
        let arr = r.point_data().get_array("MIP").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn projection_preserves_non_unit_spacing_and_centers_origin() {
        let img = ImageData::from_function(
            [2, 2, 4],
            [0.5, 2.0, 3.0],
            [10.0, 20.0, 30.0],
            "v",
            |_, _, z| z,
        );
        let r = mean_intensity_projection(&img, "v");
        assert_eq!(r.spacing(), [0.5, 2.0, 3.0]);
        assert_eq!(r.origin(), [10.0, 20.0, 34.5]);
    }

    #[test]
    fn mip_preserves_spacing_and_centers_origin() {
        let img = ImageData::from_function(
            [2, 2, 4],
            [0.5, 2.0, 3.0],
            [10.0, 20.0, 30.0],
            "v",
            |_, _, z| z,
        );
        let r = max_intensity_projection(&img, "v");
        assert_eq!(r.dimensions(), [2, 2, 1]);
        assert_eq!(r.spacing(), [0.5, 2.0, 3.0]);
        assert_eq!(r.origin(), [10.0, 20.0, 34.5]);
        let arr = r.point_data().get_array("MIP").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        // `from_function` samples at *world* coordinates, so the z values are
        // origin + k*spacing = 30, 33, 36, 39 — the projected maximum is 39,
        // not the index 3.
        assert!((buf[0] - 39.0).abs() < 1e-10);
    }

    #[test]
    fn slice_origin_tracks_selected_z() {
        let img = ImageData::from_function(
            [2, 2, 4],
            [0.5, 2.0, 3.0],
            [10.0, 20.0, 30.0],
            "v",
            |_, _, z| z,
        );
        let r = extract_z_slice(&img, "v", 2);
        assert_eq!(r.spacing(), [0.5, 2.0, 3.0]);
        assert_eq!(r.origin(), [10.0, 20.0, 36.0]);
    }
}
