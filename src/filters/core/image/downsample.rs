use crate::data::{AnyDataArray, DataArray, ImageData};

/// Downsample an ImageData by averaging blocks of voxels.
///
/// Reduces resolution by `factor` in each dimension by averaging
/// non-overlapping blocks. More robust than nearest-neighbor downsampling.
pub fn image_downsample(input: &ImageData, scalars: &str, factor: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let f = factor.max(1);
    let ncomp = arr.num_components();
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let spacing = input.spacing();
    let origin = input.origin();

    let fz = if nz <= 1 { 1 } else { f };
    let nnx = (nx / f).max(1);
    let nny = (ny / f).max(1);
    let nnz = (nz / fz).max(1);

    let mut values = vec![0.0f64; nnx * nny * nnz * ncomp];
    let mut buf = vec![0.0f64; ncomp];
    let norm = 1.0 / (f * f * fz) as f64;

    for dk in 0..nnz {
        for dj in 0..nny {
            for di in 0..nnx {
                let out_idx = ((dk * nny * nnx + dj * nnx + di) * ncomp) as usize;
                let mut sums = vec![0.0; ncomp];
                for k in dk * fz..(dk * fz + fz).min(nz) {
                    for j in dj * f..(dj * f + f).min(ny) {
                        for i in di * f..(di * f + f).min(nx) {
                            arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                            for c in 0..ncomp {
                                sums[c] += buf[c];
                            }
                        }
                    }
                }
                for c in 0..ncomp {
                    values[out_idx + c] = sums[c] * norm;
                }
            }
        }
    }

    let new_spacing = [
        spacing[0] * f as f64,
        spacing[1] * f as f64,
        spacing[2] * f as f64,
    ];
    let mut img = ImageData::with_dimensions(nnx, nny, nnz);
    img.set_origin(origin);
    img.set_spacing(new_spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, ncomp,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_2x() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        let values: Vec<f64> = (0..16).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_downsample(&img, "v", 2);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn averaging() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0, 20.0, 30.0],
                1,
            )));

        let result = image_downsample(&img, "v", 2);
        assert_eq!(result.dimensions(), [1, 1, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 15.0).abs() < 1e-10); // (0+10+20+30)/4
    }

    #[test]
    fn factor_1_noop() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = image_downsample(&img, "v", 1);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }
}
