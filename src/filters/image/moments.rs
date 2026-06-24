use crate::data::ImageData;

/// Spatial moments of a binary/scalar ImageData field.
#[derive(Debug, Clone)]
pub struct ImageMoments {
    /// Zeroth moment (total mass/volume).
    pub m000: f64,
    /// First moments (center of mass in voxel coordinates).
    pub center_x: f64,
    pub center_y: f64,
    pub center_z: f64,
    /// Second central moments (variance).
    pub var_x: f64,
    pub var_y: f64,
    pub var_z: f64,
}

/// Compute spatial moments of an ImageData scalar field.
pub fn image_moments(input: &ImageData, scalars: &str) -> Option<ImageMoments> {
    let arr = input.point_data().get_array(scalars)?;
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let sp = input.spacing();
    let origin = input.origin();

    let mut buf = [0.0f64];
    let mut m000 = 0.0;
    let mut mx = 0.0;
    let mut my = 0.0;
    let mut mz = 0.0;

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                let v = buf[0];
                let x = origin[0] + i as f64 * sp[0];
                let y = origin[1] + j as f64 * sp[1];
                let z = origin[2] + k as f64 * sp[2];
                m000 += v;
                mx += v * x;
                my += v * y;
                mz += v * z;
            }
        }
    }

    if m000.abs() < 1e-15 {
        return None;
    }
    let cx = mx / m000;
    let cy = my / m000;
    let cz = mz / m000;

    let mut vx = 0.0;
    let mut vy = 0.0;
    let mut vz = 0.0;
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                let v = buf[0];
                let x = origin[0] + i as f64 * sp[0] - cx;
                let y = origin[1] + j as f64 * sp[1] - cy;
                let z = origin[2] + k as f64 * sp[2] - cz;
                vx += v * x * x;
                vy += v * y * y;
                vz += v * z * z;
            }
        }
    }

    Some(ImageMoments {
        m000,
        center_x: cx,
        center_y: cy,
        center_z: cz,
        var_x: vx / m000,
        var_y: vy / m000,
        var_z: vz / m000,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn center_of_mass() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0, 0.0],
                1,
            )));
        let m = image_moments(&img, "v").unwrap();
        assert!((m.center_x - 1.0).abs() < 1e-10);
        assert_eq!(m.m000, 10.0);
    }

    #[test]
    fn symmetric_center() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));
        let m = image_moments(&img, "v").unwrap();
        assert!((m.center_x - 1.0).abs() < 1e-10);
        assert!((m.center_y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn zero_field() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![0.0; 9], 1)));
        assert!(image_moments(&img, "v").is_none());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        assert!(image_moments(&img, "nope").is_none());
    }
}
