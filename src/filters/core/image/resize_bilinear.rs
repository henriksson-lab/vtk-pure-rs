//! Bilinear image resizing.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resize image to new dimensions using bilinear interpolation.
pub fn resize_bilinear(
    input: &ImageData,
    scalars: &str,
    new_nx: usize,
    new_ny: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (ox, oy, oz) = (dims[0], dims[1], dims[2]);
    if ox == 0 || oy == 0 || oz == 0 {
        return input.clone();
    }
    let num_comp = arr.num_components();
    let total = ox * oy * oz;
    let mut buf = vec![0.0f64; num_comp];
    let mut vals = vec![0.0; total * num_comp];
    for i in 0..total {
        if i < arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            for c in 0..num_comp {
                vals[i * num_comp + c] = buf[c];
            }
        }
    }

    let nnx = new_nx.max(1);
    let nny = new_ny.max(1);
    let mut data = Vec::with_capacity(nnx * nny * oz * num_comp);
    for k in 0..oz {
        for iy in 0..nny {
            for ix in 0..nnx {
                let sx = ix as f64 * (ox - 1) as f64 / (nnx - 1).max(1) as f64;
                let sy = iy as f64 * (oy - 1) as f64 / (nny - 1).max(1) as f64;
                let x0 = (sx.floor() as usize).min(ox - 1);
                let x1 = (x0 + 1).min(ox - 1);
                let y0 = (sy.floor() as usize).min(oy - 1);
                let y1 = (y0 + 1).min(oy - 1);
                let fx = sx - x0 as f64;
                let fy = sy - y0 as f64;
                for c in 0..num_comp {
                    let v00 = vals[((k * oy + y0) * ox + x0) * num_comp + c];
                    let v10 = vals[((k * oy + y0) * ox + x1) * num_comp + c];
                    let v01 = vals[((k * oy + y1) * ox + x0) * num_comp + c];
                    let v11 = vals[((k * oy + y1) * ox + x1) * num_comp + c];
                    data.push(
                        v00 * (1.0 - fx) * (1.0 - fy)
                            + v10 * fx * (1.0 - fy)
                            + v01 * (1.0 - fx) * fy
                            + v11 * fx * fy,
                    );
                }
            }
        }
    }

    let sp = input.spacing();
    let new_sp = [
        if nnx > 1 {
            sp[0] * (ox - 1) as f64 / (nnx - 1) as f64
        } else {
            sp[0]
        },
        if nny > 1 {
            sp[1] * (oy - 1) as f64 / (nny - 1) as f64
        } else {
            sp[1]
        },
        sp[2],
    ];
    ImageData::with_dimensions(nnx, nny, oz)
        .with_spacing(new_sp)
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, data, num_comp,
        )))
}

/// Resize by a scale factor.
pub fn resize_by_factor(input: &ImageData, scalars: &str, factor: f64) -> ImageData {
    let dims = input.dimensions();
    let nx = (dims[0] as f64 * factor).round().max(1.0) as usize;
    let ny = (dims[1] as f64 * factor).round().max(1.0) as usize;
    resize_bilinear(input, scalars, nx, ny)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_upscale() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = resize_bilinear(&img, "v", 8, 8);
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn test_downscale() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = resize_bilinear(&img, "v", 4, 4);
        assert_eq!(r.dimensions(), [4, 4, 1]);
    }
    #[test]
    fn test_factor() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = resize_by_factor(&img, "v", 0.5);
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
