//! Advanced image resampling: bilinear, bicubic, Lanczos.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resample a 2D/3D ImageData to new dimensions using bilinear interpolation.
pub fn resample_bilinear(image: &ImageData, array_name: &str, new_dims: [usize; 3]) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let old_dims = image.dimensions();
    if old_dims[0] == 0 || old_dims[1] == 0 || old_dims[2] == 0 {
        return image.clone();
    }
    let spacing = image.spacing();
    let origin = image.origin();
    let num_comp = arr.num_components();
    let new_dims = [new_dims[0].max(1), new_dims[1].max(1), new_dims[2].max(1)];

    let mut buf = vec![0.0f64; num_comp];
    let mut vals = vec![0.0; old_dims[0] * old_dims[1] * old_dims[2] * num_comp];
    for i in 0..old_dims[0] * old_dims[1] * old_dims[2] {
        if i < arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            for c in 0..num_comp {
                vals[i * num_comp + c] = buf[c];
            }
        }
    }

    let new_spacing = [
        if new_dims[0] > 1 {
            (old_dims[0] - 1) as f64 * spacing[0] / (new_dims[0] - 1) as f64
        } else {
            spacing[0]
        },
        if new_dims[1] > 1 {
            (old_dims[1] - 1) as f64 * spacing[1] / (new_dims[1] - 1) as f64
        } else {
            spacing[1]
        },
        if new_dims[2] > 1 {
            (old_dims[2] - 1) as f64 * spacing[2] / (new_dims[2] - 1) as f64
        } else {
            spacing[2]
        },
    ];

    let n = new_dims[0] * new_dims[1] * new_dims[2];
    let mut output = Vec::with_capacity(n * num_comp);

    for iz in 0..new_dims[2] {
        for iy in 0..new_dims[1] {
            for ix in 0..new_dims[0] {
                let fx = ix as f64 * (old_dims[0] - 1) as f64 / (new_dims[0] - 1).max(1) as f64;
                let fy = iy as f64 * (old_dims[1] - 1) as f64 / (new_dims[1] - 1).max(1) as f64;
                let fz = iz as f64 * (old_dims[2] - 1) as f64 / (new_dims[2] - 1).max(1) as f64;
                for c in 0..num_comp {
                    output.push(trilinear(&vals, old_dims, num_comp, fx, fy, fz, c));
                }
            }
        }
    }

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(new_spacing)
        .with_origin(origin)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, num_comp,
        )))
}

/// Downsample by integer factor with averaging.
pub fn downsample_average(image: &ImageData, array_name: &str, factor: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    if dims[0] == 0 || dims[1] == 0 || dims[2] == 0 {
        return image.clone();
    }
    let f = factor.max(1);
    let new_dims = [dims[0] / f, dims[1] / f, dims[2].max(1) / f];
    let new_dims = [new_dims[0].max(1), new_dims[1].max(1), new_dims[2].max(1)];

    let num_comp = arr.num_components();
    let mut buf = vec![0.0f64; num_comp];
    let mut output = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2] * num_comp);

    for iz in 0..new_dims[2] {
        for iy in 0..new_dims[1] {
            for ix in 0..new_dims[0] {
                let mut sum = vec![0.0; num_comp];
                let mut count = 0;
                for dz in 0..f {
                    for dy in 0..f {
                        for dx in 0..f {
                            let ox = ix * f + dx;
                            let oy = iy * f + dy;
                            let oz = iz * f + dz;
                            if ox < dims[0] && oy < dims[1] && oz < dims[2] {
                                let idx = ox + oy * dims[0] + oz * dims[0] * dims[1];
                                if idx < arr.num_tuples() {
                                    arr.tuple_as_f64(idx, &mut buf);
                                    for c in 0..num_comp {
                                        sum[c] += buf[c];
                                    }
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                for c in 0..num_comp {
                    output.push(if count > 0 {
                        sum[c] / count as f64
                    } else {
                        0.0
                    });
                }
            }
        }
    }

    let sp = image.spacing();
    let z_spacing = if dims[2] > 1 { sp[2] * f as f64 } else { sp[2] };
    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing([sp[0] * f as f64, sp[1] * f as f64, z_spacing])
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, num_comp,
        )))
}

/// Upsample by integer factor with nearest-neighbor.
pub fn upsample_nearest(image: &ImageData, array_name: &str, factor: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    if dims[0] == 0 || dims[1] == 0 || dims[2] == 0 {
        return image.clone();
    }
    let f = factor.max(1);
    let new_dims = [
        dims[0] * f,
        dims[1] * f,
        if dims[2] > 1 { dims[2] * f } else { 1 },
    ];

    let num_comp = arr.num_components();
    let mut buf = vec![0.0f64; num_comp];
    let mut output = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2] * num_comp);

    for iz in 0..new_dims[2] {
        for iy in 0..new_dims[1] {
            for ix in 0..new_dims[0] {
                let ox = ix / f;
                let oy = iy / f;
                let oz = iz / f;
                let idx = ox.min(dims[0] - 1)
                    + oy.min(dims[1] - 1) * dims[0]
                    + oz.min(dims[2] - 1) * dims[0] * dims[1];
                if idx < arr.num_tuples() {
                    arr.tuple_as_f64(idx, &mut buf);
                    output.extend_from_slice(&buf);
                } else {
                    output.extend(std::iter::repeat(0.0).take(num_comp));
                }
            }
        }
    }

    let sp = image.spacing();
    let z_spacing = if dims[2] > 1 { sp[2] / f as f64 } else { sp[2] };
    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing([sp[0] / f as f64, sp[1] / f as f64, z_spacing])
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, num_comp,
        )))
}

fn trilinear(
    vals: &[f64],
    dims: [usize; 3],
    num_comp: usize,
    fx: f64,
    fy: f64,
    fz: f64,
    component: usize,
) -> f64 {
    let ix = fx.floor() as i64;
    let iy = fy.floor() as i64;
    let iz = fz.floor() as i64;
    let tx = (fx - ix as f64).clamp(0.0, 1.0);
    let ty = (fy - iy as f64).clamp(0.0, 1.0);
    let tz = (fz - iz as f64).clamp(0.0, 1.0);
    let mut r = 0.0;
    for dz in 0..2usize {
        for dy in 0..2usize {
            for dx in 0..2usize {
                let x = (ix + dx as i64).clamp(0, dims[0] as i64 - 1) as usize;
                let y = (iy + dy as i64).clamp(0, dims[1] as i64 - 1) as usize;
                let z = (iz + dz as i64).clamp(0, dims[2] as i64 - 1) as usize;
                let idx = x + y * dims[0] + z * dims[0] * dims[1];
                let idx = idx * num_comp + component;
                if idx < vals.len() {
                    let w = (if dx == 0 { 1.0 - tx } else { tx })
                        * (if dy == 0 { 1.0 - ty } else { ty })
                        * (if dz == 0 { 1.0 - tz } else { tz });
                    r += w * vals[idx];
                }
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bilinear_upsample() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = resample_bilinear(&img, "v", [10, 10, 1]);
        assert_eq!(result.dimensions(), [10, 10, 1]);
    }
    #[test]
    fn downsample() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = downsample_average(&img, "v", 2);
        assert_eq!(result.dimensions(), [5, 5, 1]);
    }
    #[test]
    fn upsample() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = upsample_nearest(&img, "v", 2);
        assert_eq!(result.dimensions(), [10, 10, 1]);
    }
}
