use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a 2D slice from a 3D ImageData along any axis.
///
/// axis: 0=YZ slice (fixed X), 1=XZ slice (fixed Y), 2=XY slice (fixed Z).
pub fn extract_slice_along_axis(
    input: &ImageData,
    scalars: &str,
    axis: usize,
    index: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let sp = input.spacing();
    let origin = input.origin();
    if nx == 0 || ny == 0 || nz == 0 {
        return input.clone();
    }

    let ncomp = arr.num_components();
    let mut buf = vec![0.0f64; ncomp];
    match axis.min(2) {
        0 => {
            // YZ slice at fixed X=index
            let ix = index.min(nx - 1);
            let mut values = Vec::with_capacity(ny * nz * ncomp);
            for k in 0..nz {
                for j in 0..ny {
                    arr.tuple_as_f64(k * ny * nx + j * nx + ix, &mut buf);
                    values.extend_from_slice(&buf);
                }
            }
            let mut img = ImageData::with_dimensions(ny, nz, 1);
            img.set_origin([origin[1], origin[2], origin[0] + ix as f64 * sp[0]]);
            img.set_spacing([sp[1], sp[2], 1.0]);
            img.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    scalars, values, ncomp,
                )));
            img
        }
        1 => {
            // XZ slice at fixed Y=index
            let iy = index.min(ny - 1);
            let mut values = Vec::with_capacity(nx * nz * ncomp);
            for k in 0..nz {
                for i in 0..nx {
                    arr.tuple_as_f64(k * ny * nx + iy * nx + i, &mut buf);
                    values.extend_from_slice(&buf);
                }
            }
            let mut img = ImageData::with_dimensions(nx, nz, 1);
            img.set_origin([origin[0], origin[2], origin[1] + iy as f64 * sp[1]]);
            img.set_spacing([sp[0], sp[2], 1.0]);
            img.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    scalars, values, ncomp,
                )));
            img
        }
        _ => {
            // XY slice at fixed Z=index
            let iz = index.min(nz.max(1) - 1);
            let mut values = Vec::with_capacity(nx * ny * ncomp);
            for j in 0..ny {
                for i in 0..nx {
                    arr.tuple_as_f64(iz * ny * nx + j * nx + i, &mut buf);
                    values.extend_from_slice(&buf);
                }
            }
            let mut img = ImageData::with_dimensions(nx, ny, 1);
            img.set_origin([origin[0], origin[1], origin[2] + iz as f64 * sp[2]]);
            img.set_spacing([sp[0], sp[1], 1.0]);
            img.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    scalars, values, ncomp,
                )));
            img
        }
    }
}

/// Extract maximum intensity projection along an axis.
pub fn max_intensity_projection(input: &ImageData, scalars: &str, axis: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let sp = input.spacing();
    let origin = input.origin();
    if nx == 0 || ny == 0 || nz == 0 {
        return input.clone();
    }
    let ncomp = arr.num_components();
    let mut buf = vec![0.0f64; ncomp];

    match axis.min(2) {
        0 => {
            // project along X
            let mut values = vec![f64::NEG_INFINITY; ny * nz * ncomp];
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                        let idx = k * ny + j;
                        let dst = idx * ncomp;
                        for c in 0..ncomp {
                            values[dst + c] = values[dst + c].max(buf[c]);
                        }
                    }
                }
            }
            let mut img = ImageData::with_dimensions(ny, nz, 1);
            img.set_origin([
                origin[1],
                origin[2],
                origin[0] + 0.5 * sp[0] * nx.saturating_sub(1) as f64,
            ]);
            img.set_spacing([sp[1], sp[2], 1.0]);
            img.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    scalars, values, ncomp,
                )));
            img
        }
        1 => {
            // project along Y
            let mut values = vec![f64::NEG_INFINITY; nx * nz * ncomp];
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                        let idx = k * nx + i;
                        let dst = idx * ncomp;
                        for c in 0..ncomp {
                            values[dst + c] = values[dst + c].max(buf[c]);
                        }
                    }
                }
            }
            let mut img = ImageData::with_dimensions(nx, nz, 1);
            img.set_origin([
                origin[0],
                origin[2],
                origin[1] + 0.5 * sp[1] * ny.saturating_sub(1) as f64,
            ]);
            img.set_spacing([sp[0], sp[2], 1.0]);
            img.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    scalars, values, ncomp,
                )));
            img
        }
        _ => {
            // project along Z
            let mut values = vec![f64::NEG_INFINITY; nx * ny * ncomp];
            for k in 0..nz {
                for j in 0..ny {
                    for i in 0..nx {
                        arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                        let idx = j * nx + i;
                        let dst = idx * ncomp;
                        for c in 0..ncomp {
                            values[dst + c] = values[dst + c].max(buf[c]);
                        }
                    }
                }
            }
            let mut img = ImageData::with_dimensions(nx, ny, 1);
            img.set_origin([
                origin[0],
                origin[1],
                origin[2] + 0.5 * sp[2] * nz.saturating_sub(1) as f64,
            ]);
            img.set_spacing([sp[0], sp[1], 1.0]);
            img.point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    scalars, values, ncomp,
                )));
            img
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_xy_slice() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let values: Vec<f64> = (0..27).map(|i| i as f64).collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let slice = extract_slice_along_axis(&img, "v", 2, 1); // Z=1
        assert_eq!(slice.dimensions(), [3, 3, 1]);
    }

    #[test]
    fn extract_yz_slice() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                (0..27).map(|i| i as f64).collect(),
                1,
            )));

        let slice = extract_slice_along_axis(&img, "v", 0, 1); // X=1
        assert_eq!(slice.dimensions(), [3, 3, 1]);
    }

    #[test]
    fn mip_z() {
        let mut img = ImageData::with_dimensions(2, 2, 3);
        let mut values = vec![0.0; 12];
        values[4] = 100.0; // spike at z=1
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let mip = max_intensity_projection(&img, "v", 2);
        assert_eq!(mip.dimensions(), [2, 2, 1]);
        let arr = mip.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 100.0); // max projected
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 3);
        let r = extract_slice_along_axis(&img, "nope", 2, 0);
        assert!(r.point_data().get_array("nope").is_none());
    }
}
