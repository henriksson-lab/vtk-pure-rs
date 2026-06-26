use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resample an ImageData to a new resolution using trilinear interpolation.
///
/// Creates a new ImageData with the specified dimensions that covers the same
/// spatial extent as the input. Scalar values are trilinearly interpolated.
pub fn image_resample(input: &ImageData, scalars: &str, new_dims: [usize; 3]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let old_dims = input.dimensions();
    let onx = old_dims[0] as usize;
    let ony = old_dims[1] as usize;
    let onz = old_dims[2] as usize;
    if onx == 0 || ony == 0 || onz == 0 {
        return input.clone();
    }
    let old_spacing = input.spacing();
    let origin = input.origin();

    let num_comp = arr.num_components();

    // Read old values
    let old_n = onx * ony * onz;
    let mut old_values = vec![0.0f64; old_n * num_comp];
    let mut buf = vec![0.0f64; num_comp];
    for i in 0..old_n {
        if i < arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            for c in 0..num_comp {
                old_values[i * num_comp + c] = buf[c];
            }
        }
    }

    let nnx = new_dims[0].max(1);
    let nny = new_dims[1].max(1);
    let nnz = new_dims[2].max(1);

    // Compute new spacing to cover the same extent
    let extent = [
        (onx as f64 - 1.0) * old_spacing[0],
        (ony as f64 - 1.0) * old_spacing[1],
        (onz as f64 - 1.0) * old_spacing[2],
    ];
    let new_spacing = [
        if nnx > 1 {
            extent[0] / (nnx as f64 - 1.0)
        } else {
            old_spacing[0]
        },
        if nny > 1 {
            extent[1] / (nny as f64 - 1.0)
        } else {
            old_spacing[1]
        },
        if nnz > 1 {
            extent[2] / (nnz as f64 - 1.0)
        } else {
            old_spacing[2]
        },
    ];

    let new_n = nnx * nny * nnz;
    let mut new_values = vec![0.0f64; new_n * num_comp];

    let sample = |fx: f64, fy: f64, fz: f64, component: usize| -> f64 {
        let ix = fx.floor() as i64;
        let iy = fy.floor() as i64;
        let iz = fz.floor() as i64;
        let tx = fx - ix as f64;
        let ty = fy - iy as f64;
        let tz = fz - iz as f64;

        let get = |i: i64, j: i64, k: i64| -> f64 {
            let ii = i.clamp(0, onx as i64 - 1) as usize;
            let jj = j.clamp(0, ony as i64 - 1) as usize;
            let kk = k.clamp(0, onz as i64 - 1) as usize;
            old_values[(kk * ony * onx + jj * onx + ii) * num_comp + component]
        };

        let c000 = get(ix, iy, iz);
        let c100 = get(ix + 1, iy, iz);
        let c010 = get(ix, iy + 1, iz);
        let c110 = get(ix + 1, iy + 1, iz);
        let c001 = get(ix, iy, iz + 1);
        let c101 = get(ix + 1, iy, iz + 1);
        let c011 = get(ix, iy + 1, iz + 1);
        let c111 = get(ix + 1, iy + 1, iz + 1);

        let c00 = c000 * (1.0 - tx) + c100 * tx;
        let c10 = c010 * (1.0 - tx) + c110 * tx;
        let c01 = c001 * (1.0 - tx) + c101 * tx;
        let c11 = c011 * (1.0 - tx) + c111 * tx;

        let c0 = c00 * (1.0 - ty) + c10 * ty;
        let c1 = c01 * (1.0 - ty) + c11 * ty;

        c0 * (1.0 - tz) + c1 * tz
    };

    for k in 0..nnz {
        for j in 0..nny {
            for i in 0..nnx {
                // Map new grid coords to old grid coords
                let fx = if nnx > 1 {
                    i as f64 * (onx as f64 - 1.0) / (nnx as f64 - 1.0)
                } else {
                    0.0
                };
                let fy = if nny > 1 {
                    j as f64 * (ony as f64 - 1.0) / (nny as f64 - 1.0)
                } else {
                    0.0
                };
                let fz = if nnz > 1 {
                    k as f64 * (onz as f64 - 1.0) / (nnz as f64 - 1.0)
                } else {
                    0.0
                };

                let idx = k * nny * nnx + j * nnx + i;
                for c in 0..num_comp {
                    new_values[idx * num_comp + c] = sample(fx, fy, fz, c);
                }
            }
        }
    }

    let mut img = ImageData::with_dimensions(nnx, nny, nnz);
    img.set_origin(origin);
    img.set_spacing(new_spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, new_values, num_comp,
        )));
    img.point_data_mut().set_active_scalars(scalars);
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsample() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let values: Vec<f64> = (0..27).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));

        let result = image_resample(&img, "val", [5, 5, 5]);
        assert_eq!(result.dimensions(), [5, 5, 5]);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn downsample() {
        let mut img = ImageData::with_dimensions(10, 10, 10);
        let values: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));

        let result = image_resample(&img, "val", [3, 3, 3]);
        assert_eq!(result.dimensions(), [3, 3, 3]);
    }

    #[test]
    fn preserves_corners() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 10.0, 20.0, 30.0],
                1,
            )));

        let result = image_resample(&img, "val", [3, 3, 1]);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn missing_scalars() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let result = image_resample(&img, "nope", [5, 5, 5]);
        assert_eq!(result.dimensions(), [3, 3, 3]); // unchanged
    }
}
