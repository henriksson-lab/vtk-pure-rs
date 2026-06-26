use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resample an ImageData to new dimensions using nearest-neighbor interpolation.
///
/// The output covers the same spatial extent as the input. Each output voxel
/// picks the value of the nearest input voxel.
pub fn resample_nearest(input: &ImageData, scalars: &str, new_dims: [u32; 3]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let old_dims = input.dimensions();
    let onx: usize = old_dims[0] as usize;
    let ony: usize = old_dims[1] as usize;
    let onz: usize = old_dims[2] as usize;
    if onx == 0 || ony == 0 || onz == 0 {
        return input.clone();
    }

    let nnx: usize = (new_dims[0] as usize).max(1);
    let nny: usize = (new_dims[1] as usize).max(1);
    let nnz: usize = (new_dims[2] as usize).max(1);

    let num_comp: usize = arr.num_components();

    // Read old values into flat buffer.
    let old_n: usize = onx * ony * onz;
    let mut old_values: Vec<f64> = vec![0.0; old_n * num_comp];
    let mut buf: Vec<f64> = vec![0.0; num_comp];
    for i in 0..old_n {
        if i < arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            for c in 0..num_comp {
                old_values[i * num_comp + c] = buf[c];
            }
        }
    }

    let old_spacing = input.spacing();
    let origin = input.origin();

    // Compute extent of the old grid.
    let extent_x: f64 = (onx as f64 - 1.0).max(0.0) * old_spacing[0];
    let extent_y: f64 = (ony as f64 - 1.0).max(0.0) * old_spacing[1];
    let extent_z: f64 = (onz as f64 - 1.0).max(0.0) * old_spacing[2];

    let new_spacing: [f64; 3] = [
        if nnx > 1 {
            extent_x / (nnx as f64 - 1.0)
        } else {
            old_spacing[0]
        },
        if nny > 1 {
            extent_y / (nny as f64 - 1.0)
        } else {
            old_spacing[1]
        },
        if nnz > 1 {
            extent_z / (nnz as f64 - 1.0)
        } else {
            old_spacing[2]
        },
    ];

    let new_n: usize = nnx * nny * nnz;
    let mut new_values: Vec<f64> = vec![0.0; new_n * num_comp];

    for iz in 0..nnz {
        let fz: f64 = if nnz > 1 {
            iz as f64 / (nnz as f64 - 1.0) * (onz as f64 - 1.0)
        } else {
            0.0
        };
        let oz: usize = fz.round().clamp(0.0, (onz as f64 - 1.0).max(0.0)) as usize;
        for iy in 0..nny {
            let fy: f64 = if nny > 1 {
                iy as f64 / (nny as f64 - 1.0) * (ony as f64 - 1.0)
            } else {
                0.0
            };
            let oy: usize = fy.round().clamp(0.0, (ony as f64 - 1.0).max(0.0)) as usize;
            for ix in 0..nnx {
                let fx: f64 = if nnx > 1 {
                    ix as f64 / (nnx as f64 - 1.0) * (onx as f64 - 1.0)
                } else {
                    0.0
                };
                let ox: usize = fx.round().clamp(0.0, (onx as f64 - 1.0).max(0.0)) as usize;

                let old_idx: usize = oz * ony * onx + oy * onx + ox;
                let new_idx: usize = iz * nny * nnx + iy * nnx + ix;
                for c in 0..num_comp {
                    new_values[new_idx * num_comp + c] = old_values[old_idx * num_comp + c];
                }
            }
        }
    }

    let mut out = ImageData::with_dimensions(nnx, nny, nnz);
    out.set_spacing(new_spacing);
    out.set_origin(origin);
    out.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, new_values, num_comp,
        )));
    out.point_data_mut().set_active_scalars(scalars);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, ImageData};

    #[test]
    fn upsample_2x() {
        // 2x2x1 image -> 4x4x1
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        img.set_origin([0.0, 0.0, 0.0]);
        // values: (0,0)=1  (1,0)=2  (0,1)=3  (1,1)=4
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let out = resample_nearest(&img, "s", [4, 4, 1]);
        assert_eq!(out.dimensions(), [4, 4, 1]);
        let arr = out.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        // Corner (0,0) -> nearest is (0,0)=1
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        // Corner (3,0) -> nearest is (1,0)=2
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 2.0);
    }

    #[test]
    fn downsample() {
        // 4x4x1 -> 2x2x1
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        img.set_origin([0.0, 0.0, 0.0]);
        let mut vals: Vec<f64> = vec![0.0; 16];
        for i in 0..16 {
            vals[i] = i as f64;
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));

        let out = resample_nearest(&img, "s", [2, 2, 1]);
        assert_eq!(out.dimensions(), [2, 2, 1]);
        let arr = out.point_data().get_array("s").unwrap();
        let mut buf = [0.0f64];
        // (0,0) maps to (0,0) in old = index 0 = 0.0
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        // (1,1) maps to (3,3) in old = index 15 = 15.0
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 15.0);
    }

    #[test]
    fn missing_array_returns_clone() {
        let img = ImageData::with_dimensions(2, 2, 1);
        let out = resample_nearest(&img, "missing", [4, 4, 1]);
        assert_eq!(out.dimensions(), [2, 2, 1]);
    }
}
