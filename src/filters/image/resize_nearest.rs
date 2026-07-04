use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resize an ImageData using nearest-neighbor interpolation.
///
/// Faster than trilinear resampling but produces blocky results.
pub fn image_resize_nearest(input: &ImageData, scalars: &str, new_dims: [usize; 3]) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let onx = dims[0] as usize;
    let ony = dims[1] as usize;
    let onz = dims[2] as usize;
    if onx == 0 || ony == 0 || onz == 0 {
        return input.clone();
    }
    let nnx = new_dims[0].max(1);
    let nny = new_dims[1].max(1);
    let nnz = new_dims[2].max(1);
    let num_comp = arr.num_components();
    let spacing = input.spacing();
    let origin = input.origin();

    let new_sp = [
        if nnx > 1 {
            (onx as f64 - 1.0) * spacing[0] / (nnx as f64 - 1.0)
        } else {
            spacing[0]
        },
        if nny > 1 {
            (ony as f64 - 1.0) * spacing[1] / (nny as f64 - 1.0)
        } else {
            spacing[1]
        },
        if nnz > 1 {
            (onz as f64 - 1.0) * spacing[2] / (nnz as f64 - 1.0)
        } else {
            spacing[2]
        },
    ];

    let mut buf = vec![0.0f64; num_comp];
    let mut values = Vec::with_capacity(nnx * nny * nnz * num_comp);

    for k in 0..nnz {
        for j in 0..nny {
            for i in 0..nnx {
                let oi = if onx > 1 {
                    (i as f64 * (onx - 1) as f64 / (nnx - 1).max(1) as f64).round() as usize
                } else {
                    0
                };
                let oj = if ony > 1 {
                    (j as f64 * (ony - 1) as f64 / (nny - 1).max(1) as f64).round() as usize
                } else {
                    0
                };
                let ok = if onz > 1 {
                    (k as f64 * (onz - 1) as f64 / (nnz - 1).max(1) as f64).round() as usize
                } else {
                    0
                };
                let idx = ok.min(onz - 1) * ony * onx + oj.min(ony - 1) * onx + oi.min(onx - 1);
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    values.extend_from_slice(&buf);
                } else {
                    values.extend(std::iter::repeat(0.0).take(num_comp));
                }
            }
        }
    }

    let mut img = ImageData::with_dimensions(nnx, nny, nnz);
    img.set_origin(origin);
    img.set_spacing(new_sp);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, num_comp,
        )));
    img.point_data_mut().set_active_scalars(scalars);
    img
}

/// Resize by a scale factor.
pub fn image_resize_by_factor(input: &ImageData, scalars: &str, factor: f64) -> ImageData {
    let dims = input.dimensions();
    let new_dims = [
        ((dims[0] as f64 * factor).round() as usize).max(1),
        ((dims[1] as f64 * factor).round() as usize).max(1),
        ((dims[2] as f64 * factor).round() as usize).max(1),
    ];
    image_resize_nearest(input, scalars, new_dims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upscale() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));

        let result = image_resize_nearest(&img, "v", [4, 4, 1]);
        assert_eq!(result.dimensions(), [4, 4, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // corner preserved
    }

    #[test]
    fn downscale() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                (0..16).map(|i| i as f64).collect(),
                1,
            )));

        let result = image_resize_nearest(&img, "v", [2, 2, 1]);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn by_factor() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 16],
                1,
            )));

        let result = image_resize_by_factor(&img, "v", 0.5);
        assert_eq!(result.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_resize_nearest(&img, "nope", [5, 5, 1]);
        assert_eq!(r.dimensions(), [3, 3, 1]);
    }
}
